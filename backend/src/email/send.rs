use crate::prelude::*;

use crate::email::oauth::{HTTP_CLIENT, refresh_access_token, try_load_google_secrets};
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
        "SELECT email, refresh_token FROM email_accounts
        WHERE id = $1 AND user_id = $2",
    )
    .bind(data.account_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await;
    let (from_email, refresh_token) = match account {
        Ok(row) => {
            let email: String = row.get("email");
            let refresh_token: Option<String> = row.get("refresh_token");
            (email, refresh_token)
        }
        Err(_) => return HttpResponse::Unauthorized().body("Email account not found"),
    };

    let refresh_token = match refresh_token.filter(|value| !value.trim().is_empty()) {
        Some(value) => value,
        None => {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "Reconnect Gmail account to send email"
            }));
        }
    };

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
