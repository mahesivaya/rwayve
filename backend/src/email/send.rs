use crate::prelude::*;

use crate::email::oauth::{HTTP_CLIENT, refresh_access_token, try_load_google_secrets};
use crate::email::outlook::{
    OUTLOOK_MAIL_SCOPE, outlook_credentials, refresh_outlook_token, send_outlook_mail,
};
use crate::models::email_request::SendEmailRequest;
use crate::security::jwt::get_user_id_from_request;
use actix_web::HttpResponse;
use base64::Engine;
use sqlx::PgPool;
use tracing::{error, info, instrument, warn};

#[post("/send")]
#[instrument(target = "gmail", skip(req, data, pool), fields(to = %data.to))]
pub async fn send(
    req: HttpRequest,
    data: web::Json<SendEmailRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    if data.to.trim().is_empty() || data.subject.trim().is_empty() {
        return HttpResponse::BadRequest().body("Recipient and Subject are required");
    }

    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    info!(target: "gmail", user_id, account_id = data.account_id, "send email request");

    let account = sqlx::query(
        "SELECT email, refresh_token, provider FROM email_accounts
        WHERE id = $1 AND user_id = $2",
    )
    .bind(data.account_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await;
    let (from_email, refresh_token, provider) = match account {
        Ok(row) => {
            let email: String = row.get("email");
            let refresh_token: Option<String> = row.get("refresh_token");
            let provider: String = row
                .try_get("provider")
                .unwrap_or_else(|_| "google".to_string());
            (email, refresh_token, provider)
        }
        Err(_) => return HttpResponse::Unauthorized().body("Email account not found"),
    };

    let refresh_token = match refresh_token.filter(|value| !value.trim().is_empty()) {
        Some(value) => value,
        None => {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "Reconnect the email account to send email"
            }));
        }
    };

    // Outlook mailboxes send through Microsoft Graph; Gmail continues below.
    if provider == "microsoft" {
        return send_via_outlook(pool.get_ref(), data.account_id, &refresh_token, &data).await;
    }

    let secrets = match try_load_google_secrets() {
        Ok(value) => value,
        Err(e) => {
            error!(target: "gmail", error = %e, "google secrets unavailable");
            return HttpResponse::InternalServerError().body("Google OAuth is not configured");
        }
    };
    let client_id = secrets["web"]["client_id"].as_str().unwrap_or("");
    let client_secret = secrets["web"]["client_secret"].as_str().unwrap_or("");
    let access_token = match refresh_access_token(client_id, client_secret, &refresh_token).await {
        Ok(token) => token,
        Err(e) => {
            error!(
                target: "gmail",
                user_id,
                account_id = data.account_id,
                error = ?e,
                "send token refresh failed"
            );
            return HttpResponse::BadGateway().body("Failed to refresh Gmail credentials");
        }
    };

    let _ = sqlx::query("UPDATE email_accounts SET access_token = $1 WHERE id = $2")
        .bind(&access_token)
        .bind(data.account_id)
        .execute(pool.get_ref())
        .await;

    let raw_email = format!(
        "From: {}\r\n\
    To: {}\r\n\
    Subject: {}\r\n\
    MIME-Version: 1.0\r\n\
    Content-Type: text/plain; charset=\"UTF-8\"\r\n\
    Content-Transfer-Encoding: 7bit\r\n\
    \r\n\
    {}",
        from_email.trim(),
        data.to.trim(),
        data.subject.trim(),
        data.body.replace("\n", "\r\n")
    );

    let encoded = base64::engine::general_purpose::URL_SAFE.encode(raw_email.as_bytes());

    let res = HTTP_CLIENT
        .post(crate::external::gmail_send_url())
        .bearer_auth(&access_token)
        .json(&serde_json::json!({ "raw": encoded }))
        .send()
        .await;

    match res {
        Ok(resp) => {
            let status = resp.status();
            let response_text = resp.text().await.unwrap_or_default();

            if status.is_success() {
                info!("Email sent to {} (user_id={})", data.to, user_id);
                HttpResponse::Ok().body("Email sent ✅")
            } else {
                warn!(
                    "Gmail rejected send to {} (status={}, body={})",
                    data.to, status, response_text
                );
                HttpResponse::InternalServerError()
                    .body(format!("Gmail rejected request: {}", response_text))
            }
        }
        Err(e) => {
            error!("Failed to connect to Gmail API: {}", e);
            HttpResponse::InternalServerError().body("Failed to reach Gmail")
        }
    }
}

/// Sends from a connected Outlook mailbox: refresh the Microsoft token (and
/// persist the rotated refresh token), then hand off to Graph `sendMail`.
async fn send_via_outlook(
    pool: &PgPool,
    account_id: i32,
    refresh_token: &str,
    data: &SendEmailRequest,
) -> HttpResponse {
    let creds = match outlook_credentials() {
        Some(c) => c,
        None => {
            error!(target: "gmail", "Outlook OAuth is not configured");
            return HttpResponse::InternalServerError().body("Outlook is not configured");
        }
    };

    let tokens = match refresh_outlook_token(&creds, refresh_token, OUTLOOK_MAIL_SCOPE).await {
        Ok(t) => t,
        Err(e) => {
            error!(target: "gmail", account_id, error = ?e, "outlook send token refresh failed");
            return HttpResponse::BadGateway().body("Failed to refresh Outlook credentials");
        }
    };

    // Microsoft rotates refresh tokens — persist the new one with the access token.
    let stored_refresh = tokens.refresh_token.as_deref().unwrap_or(refresh_token);
    let _ = sqlx::query(
        "UPDATE email_accounts SET access_token = $1, refresh_token = $2 WHERE id = $3",
    )
    .bind(&tokens.access_token)
    .bind(stored_refresh)
    .bind(account_id)
    .execute(pool)
    .await;

    match send_outlook_mail(
        &tokens.access_token,
        data.to.trim(),
        data.subject.trim(),
        &data.body,
    )
    .await
    {
        Ok(()) => {
            info!(target: "gmail", account_id, "outlook email sent");
            HttpResponse::Ok().body("Email sent ✅")
        }
        Err(e) => {
            error!(target: "gmail", account_id, error = ?e, "outlook send failed");
            HttpResponse::InternalServerError().body("Failed to send via Outlook")
        }
    }
}
