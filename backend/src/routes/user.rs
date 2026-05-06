use crate::models::email_request::UserResponse;
use crate::security::jwt::get_user_id_from_request;
use actix_web::{HttpRequest, HttpResponse, Responder, get, put, web};
use serde::Deserialize;
use sqlx::PgPool;

#[get("/users")]
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
            println!("DB error: {:?}", e);
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
pub async fn get_profile(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let result = sqlx::query(
        "SELECT id, email, first_name, last_name FROM users WHERE id = $1",
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

            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "email": email,
                "first_name": first_name,
                "last_name": last_name,
            }))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            println!("get_profile DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/profile")]
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

            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "email": email,
                "first_name": first_name,
                "last_name": last_name,
            }))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            println!("update_profile DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/users/all")]
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
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
