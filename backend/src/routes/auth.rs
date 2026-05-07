use crate::prelude::*;

use crate::models::auth::{LoginInput, LoginResponse, RegisterInput};
use crate::models::message::MessageResponse;
use crate::models::user::User;
use crate::security::jwt::create_jwt;
use bcrypt::{DEFAULT_COST, hash, verify};
use tracing::{error, info, instrument, warn};

#[post("/register")]
#[instrument(target = "auth", skip(pool, data), fields(email = %data.email))]
pub async fn register(pool: web::Data<PgPool>, data: web::Json<RegisterInput>) -> HttpResponse {
    info!(target: "auth", "register attempt");

    if data.password != data.confirm_password {
        warn!(target: "auth", "register rejected: password mismatch");
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "Passwords do not match" }));
    }

    let hashed = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            error!(target: "auth", error = %e, "bcrypt hash failed");
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Password hashing failed" }));
        }
    };

    let result = sqlx::query("INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id")
        .bind(&data.email)
        .bind(&hashed)
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(row) => {
            let user_id: i32 = row.get("id");
            info!(target: "auth", user_id, "User registered: {}", data.email);
            let token = create_jwt(user_id, data.email.clone());
            HttpResponse::Ok().json(serde_json::json!({ "token": token }))
        }

        Err(e) => {
            if e.to_string().contains("duplicate key") {
                warn!(target: "auth", "Register rejected (already exists): {}", data.email);
                HttpResponse::BadRequest()
                    .json(serde_json::json!({ "message": "User already exists" }))
            } else {
                error!(target: "db", error = ?e, "Register insert failed for {}", data.email);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({ "message": "Insert failed" }))
            }
        }
    }
}

#[post("/login")]
#[instrument(target = "auth", skip(pool, data), fields(email = %data.email))]
async fn login(pool: web::Data<PgPool>, data: web::Json<LoginInput>) -> HttpResponse {
    info!(target: "auth", "login attempt");

    let user_result =
        sqlx::query_as::<_, User>("SELECT id, email, password FROM users WHERE email = $1")
            .bind(&data.email)
            .fetch_optional(pool.get_ref())
            .await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!(target: "auth", "Invalid login attempt: {}", data.email);
            return HttpResponse::Unauthorized().json(MessageResponse {
                message: "Invalid credentials".to_string(),
            });
        }
        Err(e) => {
            error!(target: "db", error = ?e, "login user lookup failed");
            return HttpResponse::InternalServerError().json(MessageResponse {
                message: "Database error".to_string(),
            });
        }
    };

    let valid = match verify(&data.password, &user.password) {
        Ok(v) => v,
        Err(e) => {
            error!(target: "auth", error = %e, "bcrypt verify failed");
            return HttpResponse::InternalServerError().json(MessageResponse {
                message: "Password verification failed".to_string(),
            });
        }
    };

    if !valid {
        warn!(target: "auth", user_id = user.id, "Invalid login attempt: {}", data.email);
        return HttpResponse::Unauthorized().json(MessageResponse {
            message: "Invalid credentials".to_string(),
        });
    }

    info!(target: "auth", user_id = user.id, "Login success: {}", data.email);
    let token = create_jwt(user.id, user.email.clone());
    HttpResponse::Ok().json(LoginResponse { token })
}
