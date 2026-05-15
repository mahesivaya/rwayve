use crate::prelude::*;

use crate::security::jwt::get_user_id_from_request;
use actix_web::{HttpResponse, Responder, get};
use sqlx::PgPool;
use tracing::{error, info, instrument};

#[get("/me")]
#[instrument(target = "http", skip(req, pool))]
pub async fn get_me(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "Missing token" }));
        }
    };

    let token = auth_header.replace("Bearer ", "");

    let decoded = match crate::security::jwt::decode_jwt(&token) {
        Some(d) => d,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "Invalid token" }));
        }
    };

    let user_id = decoded.sub;

    let result =
        sqlx::query("SELECT id, email, account_type, organization_id FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool.get_ref())
            .await;

    match result {
        Ok(Some(row)) => {
            let id: i32 = row.get("id");
            let email: String = row.get("email");
            let account_type: String = row.get("account_type");
            let organization_id: Option<i32> = row.try_get("organization_id").ok().flatten();

            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "email": email,
                "account_type": account_type,
                "organization_id": organization_id
            }))
        }
        Ok(None) => {
            HttpResponse::Unauthorized().json(serde_json::json!({ "error": "User not found" }))
        }
        Err(e) => {
            error!(target: "db", error = %e, "get_me lookup failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/save-public-key")]
#[instrument(target = "auth", skip(req, pool, body))]
pub async fn save_public_key(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let public_key = body["public_key"].to_string();

    let res = sqlx::query("UPDATE users SET public_key = $1 WHERE id = $2")
        .bind(public_key)
        .bind(user_id)
        .execute(pool.get_ref())
        .await;

    match res {
        Ok(_) => {
            info!(target: "auth", user_id, "public key saved");
            HttpResponse::Ok().body("Saved")
        }
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "save_public_key failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}
