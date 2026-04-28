use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;

use actix_web::{HttpResponse, Responder, get, web, HttpRequest};
use sqlx::{PgPool, Row};

#[derive(Deserialize)]
pub struct EmailQuery {
    pub account_id: Option<i32>,
    pub before: Option<i64>,
    pub before_id: Option<i32>,
}

#[get("/emails")]
pub async fn get_emails(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<EmailQuery>,
) -> impl Responder {

    // 🔐 Extract user_id from JWT
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let limit = 50;

    let result = match (query.account_id, query.before, query.before_id) {

        // ✅ account + pagination
        (Some(account_id), Some(before), Some(before_id)) => {
            sqlx::query(
                r#"
                SELECT e.id, e.subject, e.sender, e.receiver,
                       e.body_encrypted, e.body_iv,
                       e.account_id, e.created_at
                FROM emails e
                JOIN email_accounts a ON e.account_id = a.id
                WHERE a.user_id = $1
                AND a.id = $2
                AND (e.created_at, e.id) < (to_timestamp($3), $4)
                ORDER BY e.created_at DESC, e.id DESC
                LIMIT $5
                "#
            )
            .bind(user_id)
            .bind(account_id)
            .bind(before)
            .bind(before_id)
            .bind(limit)
            .fetch_all(pool.get_ref())
            .await
        }

        // ✅ account only
        (Some(account_id), _, _) => {
            sqlx::query(
                r#"
                SELECT e.id, e.subject, e.sender, e.receiver,
                       e.body_encrypted, e.body_iv,
                       e.account_id, e.created_at
                FROM emails e
                JOIN email_accounts a ON e.account_id = a.id
                WHERE a.user_id = $1
                AND a.id = $2
                ORDER BY e.created_at DESC, e.id DESC
                LIMIT $3
                "#
            )
            .bind(user_id)
            .bind(account_id)
            .bind(limit)
            .fetch_all(pool.get_ref())
            .await
        }

        // ✅ ALL + pagination (user scoped)
        (None, Some(before), Some(before_id)) => {
            sqlx::query(
                r#"
                SELECT e.id, e.subject, e.sender, e.receiver,
                       e.body_encrypted, e.body_iv,
                       e.account_id, e.created_at
                FROM emails e
                JOIN email_accounts a ON e.account_id = a.id
                WHERE a.user_id = $1
                AND (e.created_at, e.id) < (to_timestamp($2), $3)
                ORDER BY e.created_at DESC, e.id DESC
                LIMIT $4
                "#
            )
            .bind(user_id)
            .bind(before)
            .bind(before_id)
            .bind(limit)
            .fetch_all(pool.get_ref())
            .await
        }

        // ✅ ALL default (user scoped)
        _ => {
            sqlx::query(
                r#"
                SELECT e.id, e.subject, e.sender, e.receiver,
                       e.body_encrypted, e.body_iv,
                       e.account_id, e.created_at
                FROM emails e
                JOIN email_accounts a ON e.account_id = a.id
                WHERE a.user_id = $1
                ORDER BY e.created_at DESC, e.id DESC
                LIMIT $2
                "#
            )
            .bind(user_id)
            .bind(limit)
            .fetch_all(pool.get_ref())
            .await
        }
    };

    match result {
        Ok(rows) => {
            let emails: Vec<_> = rows
                .into_iter()
                .map(|row| {
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
                })
                .collect();

            HttpResponse::Ok().json(emails)
        }
        Err(e) => {
            println!("❌ DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}