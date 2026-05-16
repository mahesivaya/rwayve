use crate::cache::Cache;
use crate::email::oauth::{load_google_secrets, refresh_access_token};
use crate::email::sync::sync_account_before;
use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;

use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::Value;
use sqlx::{PgPool, QueryBuilder, Row};
use tracing::{error, info, instrument, warn};

#[derive(Deserialize)]
pub struct EmailQuery {
    pub account_id: Option<i32>,
    pub before: Option<i64>,
    pub before_id: Option<i32>,
    pub folder: Option<String>,
    pub q: Option<String>,
}

#[derive(Deserialize)]
pub struct EmailAttachmentPath {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct EmailAttachmentDownloadPath {
    pub id: i32,
}

#[get("/emails")]
#[instrument(target = "http", skip(req, pool, _cache, query))]
pub async fn get_emails(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    _cache: web::Data<Option<Cache>>,
    query: web::Query<EmailQuery>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let page_size = 50;
    let query_limit = page_size + 1;

    if let Some(before) = query.before
        && let Err(e) = sync_older_page(
            pool.get_ref(),
            user_id,
            query.account_id,
            before,
            query_limit,
        )
        .await
    {
        warn!(target: "gmail", user_id, error = ?e, "older email page sync failed");
    }

    // 🔥 Build query dynamically
    let mut qb = QueryBuilder::new(
        r#"
        SELECT e.id, e.gmail_id, e.subject, e.sender, e.receiver,
               (e.body_encrypted <> '') AS has_body,
               EXISTS (
                   SELECT 1 FROM email_attachments ea WHERE ea.email_id = e.id
               ) AS has_attachments,
               e.account_id, e.created_at
        FROM emails e
        JOIN email_accounts a ON e.account_id = a.id
        WHERE a.user_id =
        "#,
    );

    qb.push_bind(user_id);

    // ✅ Optional account filter
    if let Some(account_id) = query.account_id {
        qb.push(" AND a.id = ");
        qb.push_bind(account_id);
    }

    // ✅ Folder filter (FIX)
    if let Some(folder) = &query.folder {
        match folder.as_str() {
            "inbox" => {
                qb.push(" AND e.receiver LIKE '%' || a.email || '%' ");
            }
            "sent" => {
                qb.push(" AND e.sender LIKE '%' || a.email || '%' ");
            }
            _ => {}
        }
    }

    if let Some(search) = query.q.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let pattern = format!("%{}%", search.to_lowercase());
        qb.push(
            r#"
            AND (
                lower(coalesce(e.subject, '')) LIKE 
            "#,
        );
        qb.push_bind(pattern.clone());
        qb.push(" OR lower(coalesce(e.sender, '')) LIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR lower(coalesce(e.receiver, '')) LIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR lower(coalesce(e.gmail_id, '')) LIKE ");
        qb.push_bind(pattern);
        qb.push(") ");
    }

    // ✅ Pagination filter
    if let (Some(before), Some(before_id)) = (query.before, query.before_id) {
        qb.push(" AND (e.created_at, e.id) < (to_timestamp(");
        qb.push_bind(before);
        qb.push("), ");
        qb.push_bind(before_id);
        qb.push(")");
    }

    // ✅ Order + limit
    qb.push(" ORDER BY e.created_at DESC, e.id DESC LIMIT ");
    qb.push_bind(query_limit as i64);

    let result = qb.build().fetch_all(pool.get_ref()).await;

    match result {
        Ok(rows) => {
            let has_more = rows.len() > page_size;
            let emails: Vec<Value> = rows
                .into_iter()
                .take(page_size)
                .map(|row| {
                    let created_at: Option<NaiveDateTime> = row.get("created_at");
                    let created_at = created_at.map(|dt| {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                            .to_rfc3339()
                    });

                    serde_json::json!({
                        "id": row.get::<i32,_>("id"),
                        "gmail_id": row.get::<String,_>("gmail_id"),
                        "subject": row.get::<Option<String>,_>("subject"),
                        "sender": row.get::<Option<String>,_>("sender"),
                        "receiver": row.get::<Option<String>,_>("receiver"),
                        "has_body": row.get::<bool,_>("has_body"),
                        "has_attachments": row.get::<bool,_>("has_attachments"),
                        "account_id": row.get::<Option<i32>,_>("account_id"),
                        "created_at": created_at,
                    })
                })
                .collect();

            info!(target: "http", user_id, count = emails.len(), "Fetched emails");
            HttpResponse::Ok()
                .append_header(("x-has-more", has_more.to_string()))
                .json(emails)
        }
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "get_emails query failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/emails/attachments")]
#[instrument(target = "http", skip(req, pool))]
pub async fn get_all_email_attachments(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let result = sqlx::query(
        r#"
        SELECT ea.id, ea.email_id, ea.filename, ea.mime_type, ea.size,
               ea.created_at, e.subject, e.sender, e.receiver
        FROM email_attachments ea
        JOIN emails e ON ea.email_id = e.id
        JOIN email_accounts a ON ea.account_id = a.id
        WHERE a.user_id = $1
        ORDER BY ea.created_at DESC, ea.id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let files: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    let created_at: Option<NaiveDateTime> = row.get("created_at");
                    let created_at = created_at.map(|dt| {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                            .to_rfc3339()
                    });

                    serde_json::json!({
                        "id": row.get::<i32, _>("id"),
                        "email_id": row.get::<i32, _>("email_id"),
                        "filename": row.get::<String, _>("filename"),
                        "mime_type": row.get::<Option<String>, _>("mime_type"),
                        "size": row.get::<Option<i64>, _>("size"),
                        "created_at": created_at,
                        "subject": row.get::<Option<String>, _>("subject"),
                        "sender": row.get::<Option<String>, _>("sender"),
                        "receiver": row.get::<Option<String>, _>("receiver"),
                    })
                })
                .collect();

            HttpResponse::Ok().json(files)
        }
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "get_all_email_attachments failed");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch email attachments"
            }))
        }
    }
}

#[get("/emails/{id}/attachments")]
#[instrument(target = "http", skip(req, pool, path))]
pub async fn get_email_attachments(
    req: HttpRequest,
    path: web::Path<EmailAttachmentPath>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let email_id = path.id;

    let result = sqlx::query(
        r#"
        SELECT ea.id, ea.email_id, ea.filename, ea.mime_type, ea.size, ea.created_at
        FROM email_attachments ea
        JOIN email_accounts a ON ea.account_id = a.id
        WHERE ea.email_id = $1 AND a.user_id = $2
        ORDER BY ea.id ASC
        "#,
    )
    .bind(email_id)
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let files: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    let created_at: Option<NaiveDateTime> = row.get("created_at");
                    let created_at = created_at.map(|dt| {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                            .to_rfc3339()
                    });

                    serde_json::json!({
                        "id": row.get::<i32, _>("id"),
                        "email_id": row.get::<i32, _>("email_id"),
                        "filename": row.get::<String, _>("filename"),
                        "mime_type": row.get::<Option<String>, _>("mime_type"),
                        "size": row.get::<Option<i64>, _>("size"),
                        "created_at": created_at,
                    })
                })
                .collect();

            HttpResponse::Ok().json(files)
        }
        Err(e) => {
            error!(target: "db", user_id, email_id, error = ?e, "get_email_attachments failed");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch email attachments"
            }))
        }
    }
}

#[get("/email-attachments/{id}/download")]
#[instrument(target = "gmail", skip(req, pool, path))]
pub async fn download_email_attachment(
    req: HttpRequest,
    path: web::Path<EmailAttachmentDownloadPath>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let attachment_row = sqlx::query(
        r#"
        SELECT ea.attachment_id, ea.gmail_id, ea.filename, ea.mime_type,
               a.id AS account_id, a.refresh_token, a.provider
        FROM email_attachments ea
        JOIN email_accounts a ON ea.account_id = a.id
        WHERE ea.id = $1 AND a.user_id = $2
        "#,
    )
    .bind(path.id)
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await;

    let row = match attachment_row {
        Ok(Some(row)) => row,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(target: "db", user_id, attachment_id = path.id, error = ?e, "attachment lookup failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let account_id: i32 = row.get("account_id");
    let refresh_token: Option<String> = row.get("refresh_token");
    let refresh_token = match refresh_token.filter(|value| !value.trim().is_empty()) {
        Some(value) => value,
        None => {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "Reconnect Gmail account to download this attachment"
            }));
        }
    };

    let provider: String = row
        .try_get("provider")
        .unwrap_or_else(|_| "google".to_string());
    let gmail_id: String = row.get("gmail_id");
    let gmail_attachment_id: String = row.get("attachment_id");
    let filename: String = row.get("filename");
    let mime_type: Option<String> = row.get("mime_type");

    // Outlook attachments come from Microsoft Graph; Gmail continues below.
    if provider == "microsoft" {
        return download_outlook_attachment(
            pool.get_ref(),
            OutlookAttachmentRef {
                account_id,
                refresh_token: &refresh_token,
                message_id: &gmail_id,
                attachment_id: &gmail_attachment_id,
                filename: &filename,
                mime_type,
            },
        )
        .await;
    }

    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"].as_str().unwrap_or("");
    let client_secret = secrets["web"]["client_secret"].as_str().unwrap_or("");
    let token = match refresh_access_token(client_id, client_secret, &refresh_token).await {
        Ok(token) => token,
        Err(e) => {
            error!(target: "gmail", account_id, error = ?e, "attachment token refresh failed");
            return HttpResponse::BadGateway().finish();
        }
    };

    let url = format!(
        "{}/gmail/v1/users/me/messages/{}/attachments/{}",
        crate::external::gmail_api_base(),
        gmail_id,
        gmail_attachment_id
    );

    let res: Value = match crate::email::oauth::HTTP_CLIENT
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(resp) => match resp.json().await {
            Ok(json) => json,
            Err(e) => {
                error!(target: "gmail", error = %e, "attachment json parse failed");
                return HttpResponse::BadGateway().finish();
            }
        },
        Err(e) => {
            error!(target: "gmail", error = %e, "attachment request failed");
            return HttpResponse::BadGateway().finish();
        }
    };

    let data = res["data"].as_str().unwrap_or("");
    let bytes = match URL_SAFE_NO_PAD.decode(data) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!(target: "gmail", error = ?e, "attachment base64 decode failed");
            return HttpResponse::BadGateway().finish();
        }
    };

    HttpResponse::Ok()
        .insert_header((
            header::CONTENT_TYPE,
            mime_type.unwrap_or_else(|| "application/octet-stream".to_string()),
        ))
        .insert_header((
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename.replace('"', "")),
        ))
        .body(bytes)
}

/// A stored Outlook attachment to download — bundles the values clippy's
/// argument-count cap would otherwise reject as a long parameter list.
struct OutlookAttachmentRef<'a> {
    account_id: i32,
    refresh_token: &'a str,
    message_id: &'a str,
    attachment_id: &'a str,
    filename: &'a str,
    mime_type: Option<String>,
}

/// Downloads an Outlook attachment's bytes via Microsoft Graph, refreshing the
/// mailbox token first (and persisting the rotated refresh token).
async fn download_outlook_attachment(
    pool: &PgPool,
    att: OutlookAttachmentRef<'_>,
) -> HttpResponse {
    let creds = match crate::email::outlook::outlook_credentials() {
        Some(c) => c,
        None => {
            error!(target: "gmail", "Outlook OAuth is not configured");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let tokens = match crate::email::outlook::refresh_outlook_token(
        &creds,
        att.refresh_token,
        crate::email::outlook::OUTLOOK_MAIL_SCOPE,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            error!(target: "gmail", account_id = att.account_id, error = ?e, "outlook attachment token refresh failed");
            return HttpResponse::BadGateway().finish();
        }
    };

    let stored_refresh = tokens.refresh_token.as_deref().unwrap_or(att.refresh_token);
    let _ = sqlx::query(
        "UPDATE email_accounts SET access_token = $1, refresh_token = $2 WHERE id = $3",
    )
    .bind(&tokens.access_token)
    .bind(stored_refresh)
    .bind(att.account_id)
    .execute(pool)
    .await;

    let (content_type, bytes) = match crate::email::outlook::fetch_outlook_attachment_bytes(
        &tokens.access_token,
        att.message_id,
        att.attachment_id,
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            error!(target: "gmail", account_id = att.account_id, error = ?e, "outlook attachment download failed");
            return HttpResponse::BadGateway().finish();
        }
    };

    HttpResponse::Ok()
        .insert_header((
            header::CONTENT_TYPE,
            att.mime_type
                .or(content_type)
                .unwrap_or_else(|| "application/octet-stream".to_string()),
        ))
        .insert_header((
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", att.filename.replace('"', "")),
        ))
        .body(bytes)
}

async fn sync_older_page(
    pool: &PgPool,
    user_id: i32,
    account_id: Option<i32>,
    before_timestamp: i64,
    limit: usize,
) -> anyhow::Result<()> {
    let mut qb = QueryBuilder::new("SELECT id, refresh_token FROM email_accounts WHERE user_id = ");
    qb.push_bind(user_id);

    if let Some(account_id) = account_id {
        qb.push(" AND id = ");
        qb.push_bind(account_id);
    }

    let rows = qb.build().fetch_all(pool).await?;
    if rows.is_empty() {
        return Ok(());
    }

    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"].as_str().unwrap_or("");
    let client_secret = secrets["web"]["client_secret"].as_str().unwrap_or("");

    for row in rows {
        let account_id: i32 = row.get("id");
        let refresh_token: String = row.get("refresh_token");
        let token = refresh_access_token(client_id, client_secret, &refresh_token).await?;

        let _ = sqlx::query("UPDATE email_accounts SET access_token = $1 WHERE id = $2")
            .bind(&token)
            .bind(account_id)
            .execute(pool)
            .await;

        sync_account_before(pool, account_id, &token, before_timestamp, limit).await?;
    }

    Ok(())
}
