use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::{PgPool, Row, QueryBuilder};

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
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let limit = 50;

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
    qb.push_bind(limit);

    let result = qb.build().fetch_all(pool.get_ref()).await;

    match result {
        Ok(rows) => {
            let emails: Vec<_> = rows
                .into_iter()
                .map(|row| {
                    serde_json::json!({
                        "id": row.get::<i32,_>("id"),
                        "gmail_id": row.get::<String,_>("gmail_id"),
                        "subject": row.get::<Option<String>,_>("subject"),
                        "sender": row.get::<Option<String>,_>("sender"),
                        "receiver": row.get::<Option<String>,_>("receiver"),
                        "has_body": row.get::<bool,_>("has_body"),
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