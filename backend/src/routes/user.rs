use crate::models::auth::ChangePasswordInput;
use crate::models::email_request::UserResponse;
use crate::security::jwt::get_user_id_from_request;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, put, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{error, info, instrument, warn};

#[get("/users")]
#[instrument(target = "http", skip(req, pool))]
pub async fn get_user_by_email(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let query = req.query_string();

    let email = match query.split("email=").nth(1) {
        Some(e) => e,
        None => return HttpResponse::BadRequest().body("Email required"),
    };

    let result = sqlx::query_as::<_, UserResponse>(
        "SELECT id, email, public_key FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => {
            let parsed_key = user
                .public_key
                .and_then(|k| serde_json::from_str::<Vec<u8>>(&k).ok());

            HttpResponse::Ok().json(serde_json::json!({
                "id": user.id,
                "email": user.email,
                "public_key": parsed_key
            }))
        }

        Ok(None) => HttpResponse::Ok().json(serde_json::json!(null)),

        Err(e) => {
            error!(target: "db", error = ?e, "get_user_by_email lookup failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

use sqlx::Row;

#[derive(Deserialize)]
pub struct ProfileUpdate {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[get("/profile")]
#[instrument(target = "http", skip(req, pool))]
pub async fn get_profile(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let result = sqlx::query(
        "SELECT id, email, first_name, last_name, auth_provider FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let id: i32 = row.get("id");
            let email: String = row.get("email");
            let first_name: Option<String> = row.try_get("first_name").ok();
            let last_name: Option<String> = row.try_get("last_name").ok();
            let auth_provider: String = row.get("auth_provider");

            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "email": email,
                "first_name": first_name,
                "last_name": last_name,
                "auth_provider": auth_provider,
            }))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "get_profile lookup failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/profile/password")]
#[instrument(target = "auth", skip(req, pool, data))]
pub async fn change_password(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<ChangePasswordInput>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if data.new_password.len() < 6 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "New password must be at least 6 characters" }));
    }

    let row = sqlx::query("SELECT password, auth_provider FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool.get_ref())
        .await;

    let (stored, auth_provider): (Option<String>, String) = match row {
        Ok(Some(r)) => (
            r.try_get("password").ok().flatten(),
            r.try_get("auth_provider")
                .unwrap_or_else(|_| "local".to_string()),
        ),
        _ => return HttpResponse::Unauthorized().finish(),
    };

    if let Some(stored) = stored {
        let current_password = data.current_password.as_deref().unwrap_or("");
        let valid = verify(current_password, &stored).unwrap_or(false);
        if !valid {
            warn!(target: "auth", user_id, "change-password: wrong current password");
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "message": "Current password is incorrect" }));
        }
    } else if auth_provider != "google" {
        warn!(target: "auth", user_id, "change-password rejected: missing password");
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "This account has no password to change" }));
    }

    let hashed = match hash(&data.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            error!(target: "auth", error = %e, "bcrypt hash failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Err(e) = sqlx::query("UPDATE users SET password = $1 WHERE id = $2")
        .bind(&hashed)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
    {
        error!(target: "auth", user_id, error = %e, "password update failed");
        return HttpResponse::InternalServerError().finish();
    }

    info!(target: "auth", user_id, had_password = data.current_password.is_some(), "password updated");
    HttpResponse::Ok().json(serde_json::json!({ "message": "Password updated" }))
}

#[put("/profile")]
#[instrument(target = "http", skip(req, pool, data))]
pub async fn update_profile(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<ProfileUpdate>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let result = sqlx::query(
        "UPDATE users
         SET first_name = $1, last_name = $2
         WHERE id = $3
         RETURNING id, email, first_name, last_name",
    )
    .bind(data.first_name.as_deref().unwrap_or(""))
    .bind(data.last_name.as_deref().unwrap_or(""))
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let id: i32 = row.get("id");
            let email: String = row.get("email");
            let first_name: Option<String> = row.try_get("first_name").ok();
            let last_name: Option<String> = row.try_get("last_name").ok();

            info!(target: "http", user_id, "profile updated");
            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "email": email,
                "first_name": first_name,
                "last_name": last_name,
            }))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "update_profile failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/users/all")]
#[instrument(target = "http", skip(pool))]
async fn get_all_users(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query("SELECT id, email FROM users")
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(rows) => {
            let users: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    let id: i32 = r.get("id");
                    let email: String = r.get("email");

                    serde_json::json!({
                        "id": id,
                        "email": email
                    })
                })
                .collect();

            HttpResponse::Ok().json(users)
        }
        Err(e) => {
            error!(target: "db", error = ?e, "get_all_users failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}
