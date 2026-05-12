use crate::cache::Cache;
use crate::email::oauth::{load_google_secrets, refresh_access_token};
use crate::email::sync::sync_account_before;
use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;

use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use serde_json::Value;
use sqlx::{PgPool, QueryBuilder, Row};
use tracing::{error, info, instrument, warn};

#[derive(Deserialize)]
pub struct EmailQuery {
    pub account_id: Option<i32>,
    pub before: Option<i64>,
    pub before_id: Option<i32>,
    pub folder: Option<String>,
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
