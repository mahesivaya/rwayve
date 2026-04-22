use crate::prelude::*;

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use sqlx::Row;


#[derive(Serialize)]
pub struct EmailResponse {
    pub id: i32,
    pub subject: Option<String>,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub body_encrypted: String,
    pub body_iv: String,
    pub account_id: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
}


#[derive(Deserialize)]
pub struct EmailQuery {
    pub account_id: Option<i32>,
    pub before: Option<i64>,
    pub before_id: Option<i32>,
}


#[get("/emails")]
pub async fn get_emails(
    pool: web::Data<PgPool>,
    query: web::Query<EmailQuery>,
) -> impl Responder {

    let limit = 50;

    let result = match (query.account_id, query.before, query.before_id) {

        // 🔥 account + pagination
        (Some(account_id), Some(before), Some(before_id)) => {
            sqlx::query(
                r#"
                SELECT id, subject, sender, receiver, body_encrypted, body_iv, account_id, created_at
                FROM emails
                WHERE account_id = $1
                AND (created_at, id) < (to_timestamp($2), $3)
                ORDER BY created_at DESC, id DESC
                LIMIT $4
                "#
            )
            .bind(account_id)
            .bind(before)
            .bind(before_id)
            .bind(limit)
            .fetch_all(pool.get_ref())
            .await
        }

        // 🔥 account only
        (Some(account_id), _, _) => {
            sqlx::query(
                r#"
                SELECT id, subject, sender, receiver, body_encrypted, body_iv, account_id, created_at
                FROM emails
                WHERE account_id = $1
                ORDER BY created_at DESC, id DESC
                LIMIT $2
                "#
            )
            .bind(account_id)
            .bind(limit)
            .fetch_all(pool.get_ref())
            .await
        }

        // 🔥 ALL + pagination
        (None, Some(before), Some(before_id)) => {
            sqlx::query(
                r#"
                SELECT id, subject, sender, receiver, body_encrypted, body_iv, account_id, created_at
                FROM emails
                WHERE (created_at, id) < (to_timestamp($1), $2)
                ORDER BY created_at DESC, id DESC
                LIMIT $3
                "#
            )
            .bind(before)
            .bind(before_id)
            .bind(limit)
            .fetch_all(pool.get_ref())
            .await
        }

        // 🔥 ALL default
        _ => {
            sqlx::query(
                r#"
                SELECT id, subject, sender, receiver, body_encrypted, body_iv, account_id, created_at
                FROM emails
                ORDER BY created_at DESC, id DESC
                LIMIT $1
                "#
            )
            .bind(limit)
            .fetch_all(pool.get_ref())
            .await
        }
    };

    match result {
        Ok(rows) => {
            let emails: Vec<_> = rows.into_iter().map(|row| {
                serde_json::json!({
                    "id": row.get::<i32,_>("id"),
                    "subject": row.get::<Option<String>,_>("subject"),
                    "sender": row.get::<Option<String>,_>("sender"),
                    "receiver": row.get::<Option<String>,_>("receiver"),
                    "body_encrypted": row.get::<String,_>("body_encrypted"),
                    "body_iv": row.get::<String,_>("body_iv"),
                    "account_id": row.get::<Option<i32>,_>("account_id"),
                    "created_at": row.get::<Option<NaiveDateTime>,_>("created_at"),
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