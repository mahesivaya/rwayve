use crate::prelude::*;

pub async fn send_email(
    pool: &PgPool,
    account_id: i32,
    to: String,
    subject: String,
    body: String,
) -> Result<(), String> {

    let row = sqlx::query(
        "SELECT email, access_token FROM email_accounts WHERE id = $1"
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(|_| "Email account not found".to_string())?;

    let from_email: String = row.get("email");
    let access_token: String = row.get("access_token");

    let raw_email = format!(
        "From: {}\r\n\
To: {}\r\n\
Subject: {}\r\n\
MIME-Version: 1.0\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
{}",
        from_email.trim(),
        to.trim(),
        subject.trim(),
        body
    );

    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(raw_email.as_bytes());

    let client = reqwest::Client::new();

    let res = client
        .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
        .bearer_auth(&access_token)
        .json(&serde_json::json!({ "raw": encoded }))
        .send()
        .await
        .map_err(|_| "Failed to reach Gmail".to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let text = res.text().await.unwrap_or_default();
        Err(format!("Gmail rejected request: {}", text))
    }
}