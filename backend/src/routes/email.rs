use crate::prelude::*;

use crate::security::encryption::decrypt;

#[derive(serde::Deserialize, Default)]
pub struct EmailQuery {
    #[serde(default)]
    pub account_id: Option<i32>,

    #[serde(default)]
    pub before: Option<NaiveDateTime>, // 🔥 MUST BE STRING

    #[serde(default)]
    pub before_id: Option<i32>,
}


#[get("/emails")]
async fn get_emails(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<EmailQuery>,
) -> impl Responder {

    let token = match req.headers().get("Authorization") {
        Some(h) => {
            let val = h.to_str().unwrap_or("");
            if !val.starts_with("Bearer ") {
                return HttpResponse::Unauthorized().body("Invalid token format");
            }
            val.trim_start_matches("Bearer ").to_string()
        }
        None => return HttpResponse::Unauthorized().body("Missing token"),
    };

    let decoded = match crate::models::auth::decode_jwt(&token) {
        Some(d) => d,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = decoded.sub;

    let query = query.into_inner();

    let result = if let Some(before_time) = query.before {
        let before_id = query.before_id.unwrap_or(i32::MAX);

        if let Some(account_id) = query.account_id {
            // ✅ PAGINATION (SPECIFIC ACCOUNT)
            sqlx::query(
                r#"
                SELECT 
                    e.id, e.sender, e.subject, e.body_encrypted, e.body_iv, e.created_at
                FROM emails e
                JOIN email_accounts a ON e.account_id = a.id
                WHERE e.account_id = $1
                AND a.user_id = $2
                ORDER BY e.created_at DESC, e.id DESC
                LIMIT 50
                "#
            )
            .bind(account_id)
            .bind(user_id)
            .bind(before_time)
            .bind(before_id)
            .fetch_all(pool.get_ref())
            .await

        } else {
            // ✅ PAGINATION (ALL EMAILS)
            sqlx::query(
                r#"
                SELECT 
                    e.id, e.sender, e.subject, e.body_encrypted, e.body_iv, e.created_at
                FROM emails e
                JOIN email_accounts a ON e.account_id = a.id
                WHERE a.user_id = $1
                AND (
                    e.created_at < $2
                    OR (e.created_at = $2 AND e.id < $3)
                )
                ORDER BY e.created_at DESC, e.id DESC
                LIMIT 50
                "#
            )
            .bind(user_id)
            .bind(before_time)
            .bind(before_id)
            .fetch_all(pool.get_ref())
            .await
        }

    } else {
        if let Some(account_id) = query.account_id {
            // ✅ INITIAL LOAD (SPECIFIC ACCOUNT)
            sqlx::query(
                r#"
                SELECT 
                    e.id, e.sender, e.subject, e.body_encrypted, e.body_iv, e.created_at
                FROM emails e
                JOIN email_accounts a ON e.account_id = a.id
                WHERE a.user_id = $1
                ORDER BY e.created_at DESC, e.id DESC
                LIMIT 50
                "#
            )
            .bind(account_id)
            .bind(user_id)
            .fetch_all(pool.get_ref())
            .await
        } else {
            // ✅ INITIAL LOAD (ALL EMAILS)
            sqlx::query(
                r#"
                SELECT 
                    e.id,
                    e.sender,
                    e.subject,
                    e.body_encrypted,
                    e.body_iv,
                    e.created_at
                FROM emails e
                JOIN email_accounts a ON e.account_id = a.id
                WHERE a.user_id = $1
                ORDER BY e.created_at DESC, e.id DESC
                LIMIT 50
                "#
            )
            .bind(user_id)
            .fetch_all(pool.get_ref())
            .await
        }
    };

    match result {
        Ok(rows) => {
            let emails: Vec<_> = rows.into_iter().map(|row| {
                let id: i32 = row.try_get("id").unwrap_or(0);
                let sender: String = row.try_get("sender").unwrap_or_default();
                let subject: String = row.try_get("subject").unwrap_or_default();
                let created_at: chrono::NaiveDateTime =
                    row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now().naive_utc());

                let iv: Option<String> = row.try_get("body_iv").ok();
                let enc: Option<String> = row.try_get("body_encrypted").ok();

                let plain_body: Option<String> = row.try_get("body").ok();

                let body = if let (Some(iv), Some(enc)) = (iv, enc) {
                    let decrypted = decrypt(&iv, &enc);

                    if decrypted.is_empty() {
                        plain_body.unwrap_or_default()  // 🔥 fallback
                    } else {
                        decrypted
                    }
                } else {
                    plain_body.unwrap_or_default() // 🔥 fallback
                };

                serde_json::json!({
                    "id": id,
                    "sender": sender,
                    "subject": subject,
                    "body": body,
                    "created_at": created_at
                })
            }).collect();

            HttpResponse::Ok().json(emails)
        }

        Err(e) => {
            println!("❌ DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}