use crate::email::account::{
    invalidate_email_account_cache, invalidate_user_account_list_cache,
    load_account_summaries_for_user,
};
use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;
use actix_web::{delete, put};
use serde::Deserialize;
use tracing::{error, info, instrument};

#[get("/accounts")]
#[instrument(target = "http", skip(req, pool))]
pub(crate) async fn get_accounts(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Missing or invalid token"),
    };

    let result = load_account_summaries_for_user(pool.get_ref(), user_id).await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "get_accounts failed");
            HttpResponse::InternalServerError().body("error")
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateAccountNameBody {
    display_name: Option<String>,
}

#[put("/accounts/{id}/display-name")]
#[instrument(target = "http", skip(req, pool, body))]
pub async fn update_account_display_name(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    body: web::Json<UpdateAccountNameBody>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let id = path.into_inner();
    let display_name = body
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let result = sqlx::query(
        r#"
        UPDATE email_accounts
        SET display_name = $1
        WHERE id = $2 AND user_id = $3
        "#,
    )
    .bind(display_name)
    .bind(id)
    .bind(user_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Email account not found"
        })),
        Ok(_) => {
            invalidate_user_account_list_cache(user_id).await;
            HttpResponse::Ok().json(serde_json::json!({ "updated": true }))
        }
        Err(e) => {
            error!(target: "db", user_id, account_id = id, error = ?e, "update_account_display_name failed");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update email account name"
            }))
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
            invalidate_email_account_cache(id).await;
            invalidate_user_account_list_cache(user_id).await;
            info!("Email account deleted: id={} user_id={}", id, user_id);
            HttpResponse::Ok().json(serde_json::json!({ "deleted": true }))
        }
        Err(e) => {
            error!(target: "db", user_id, account_id = id, error = ?e, "delete_account failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}
