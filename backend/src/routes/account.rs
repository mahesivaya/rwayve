use crate::models::account::Account;
use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;
use actix_web::delete;

#[get("/accounts")]
async fn get_accounts(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    // 🔥 Extract token
    let token = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or("").replace("Bearer ", ""),
        None => return HttpResponse::Unauthorized().body("Missing token"),
    };

    // 🔥 Decode JWT
    let decoded = match crate::security::jwt::decode_jwt(&token) {
        Some(d) => d,
        None => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let user_id = decoded.sub;

    // ✅ Filter by user_id
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
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("error")
        }
    }
}

#[delete("/accounts/{id}")]
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

    // Owner-scoped: 404 if the row doesn't exist or belongs to someone else.
    // Cascading FKs on emails(account_id) clean up the synced messages.
    let result = sqlx::query("DELETE FROM email_accounts WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => HttpResponse::NotFound().finish(),
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "deleted": true })),
        Err(e) => {
            println!("delete_account DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
