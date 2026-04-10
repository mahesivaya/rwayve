use crate::prelude::*;
use crate::decrypt;

#[derive(serde::Deserialize, Default)]
pub struct EmailQuery {
    #[serde(default)]
    pub account_id: Option<i32>,

    #[serde(default)]
    pub before: Option<NaiveDateTime>, // 🔥 MUST BE STRING

    #[serde(default)]
    pub before_id: Option<i32>,
}


#[get("/api/emails")]
pub async fn get_emails(
    pool: web::Data<PgPool>,
    query: web::Query<EmailQuery>,
) -> impl Responder {

    let query = query.into_inner();

    let result = if let Some(before_time) = query.before {
        let before_id = query.before_id.unwrap_or(i32::MAX);

        if let Some(account_id) = query.account_id {
            // ✅ PAGINATION (SPECIFIC ACCOUNT)
            sqlx::query(
                r#"
                SELECT id, sender, subject, body_encrypted, body_iv, created_at
                FROM emails
                WHERE account_id = $1
                AND (
                    created_at < $2::timestamp
                    OR (created_at = $2::timestamp AND id < $3)
                )
                ORDER BY created_at DESC, id DESC
                LIMIT 50
                "#
            )
            .bind(account_id)
            .bind(before_time)
            .bind(before_id)
            .fetch_all(pool.get_ref())
            .await

        } else {
            // ✅ PAGINATION (ALL EMAILS)
            sqlx::query(
                r#"
                SELECT id, sender, subject, body_encrypted, body_iv, created_at
                FROM emails
                WHERE (
                    created_at < $1::timestamp
                    OR (created_at = $1::timestamp AND id < $2)
                )
                ORDER BY created_at DESC, id DESC
                LIMIT 50
                "#
            )
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
                SELECT id, sender, subject, body_encrypted, body_iv, created_at
                FROM emails
                WHERE account_id = $1
                ORDER BY created_at DESC, id DESC
                LIMIT 50
                "#
            )
            .bind(account_id)
            .fetch_all(pool.get_ref())
            .await
        } else {
            // ✅ INITIAL LOAD (ALL EMAILS)
            sqlx::query(
                r#"
                SELECT id, sender, subject, body_encrypted, body_iv, created_at
                FROM emails
                ORDER BY created_at DESC, id DESC
                LIMIT 50
                "#
            )
            .fetch_all(pool.get_ref())
            .await
        }
    };

    match result {
        Ok(rows) => {
            let emails: Vec<_> = rows.into_iter().map(|row| {
                let id: i32 = row.try_get("id").unwrap();
                let sender: String = row.try_get("sender").unwrap();
                let subject: String = row.try_get("subject").unwrap();
                let created_at: chrono::NaiveDateTime =
                    row.try_get("created_at").unwrap();

                let iv: Option<String> = row.try_get("body_iv").ok();
                let enc: Option<String> = row.try_get("body_encrypted").ok();

                let body = if let (Some(iv), Some(enc)) = (iv, enc) {
                    decrypt(&iv, &enc)
                } else {
                    "".to_string()
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