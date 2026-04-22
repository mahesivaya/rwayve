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
}

#[get("/emails")]
pub async fn get_emails(
    pool: web::Data<PgPool>,
    query: web::Query<EmailQuery>,
) -> impl Responder {

    let result = if let Some(account_id) = query.account_id {

        sqlx::query(
            r#"
            SELECT id, subject, sender, receiver, body_encrypted, body_iv, account_id, created_at
            FROM emails
            WHERE account_id = $1
            ORDER BY created_at DESC
            LIMIT 50
            "#
        )
        .bind(account_id)
        .fetch_all(pool.get_ref())
        .await

    } else {

        sqlx::query(
            r#"
            SELECT id, subject, sender, receiver, body_encrypted, body_iv, account_id, created_at
            FROM emails
            ORDER BY created_at DESC
            LIMIT 50
            "#
        )
        .fetch_all(pool.get_ref())
        .await
    };

    match result {
        Ok(rows) => {

            let emails: Vec<EmailResponse> = rows.into_iter().map(|row| {
                EmailResponse {
                    id: row.get("id"),
                    subject: row.get("subject"),
                    sender: row.get("sender"),
                    receiver: row.get("receiver"),
                    body_encrypted: row.get("body_encrypted"),
                    body_iv: row.get("body_iv"),
                    account_id: row.get("account_id"),
                    created_at: row.get("created_at"),
                }
            }).collect();

            HttpResponse::Ok().json(emails)
        }

        Err(e) => {
            println!("❌ DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}