use crate::models::email_request::UserResponse;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
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
