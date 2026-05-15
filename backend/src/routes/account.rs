use crate::models::account::Account;
use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;
use actix_web::delete;
use tracing::{error, info, instrument};

#[get("/accounts")]
#[instrument(target = "http", skip(req, pool))]
pub(crate) async fn get_accounts(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Missing or invalid token"),
    };

    let result = sqlx::query_as::<_, Account>(
        r#"
        SELECT id, email
        FROM email_accounts
        WHERE user_id = $1
        ORDER BY id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "get_accounts failed");
            HttpResponse::InternalServerError().body("error")
        }
    }
}

#[delete("/accounts/{id}")]
#[instrument(target = "http", skip(req, pool))]
pub async fn delete_account(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let id = path.into_inner();

    let result = sqlx::query("DELETE FROM email_accounts WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => HttpResponse::NotFound().finish(),
        Ok(_) => {
            info!("Email account deleted: id={} user_id={}", id, user_id);
            HttpResponse::Ok().json(serde_json::json!({ "deleted": true }))
        }
        Err(e) => {
            error!(target: "db", user_id, account_id = id, error = ?e, "delete_account failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}
