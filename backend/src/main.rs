// ==============================
// 🔹 INTERNAL MODULES (declare first)
// ==============================
mod prelude;
mod models;
mod chat;
mod gmail;
mod scheduler;
mod drive;
pub mod security;
mod call;
mod logging;



// ==============================
// 🔹 USE INTERNAL MODULES
// ==============================
use crate::prelude::*;
use crate::logging::logger::init_logger;
use crate::chat::{chat_ws, get_messages};
use crate::gmail::{gmail_login, oauth_callback, send};
use crate::drive::{upload_file, get_files};
use crate::scheduler::{create_meeting, get_meetings};
use crate::call::call::call_ws;
use crate::security::encryption::decrypt;


// ==============================
// 🔹 EXTERNAL CRATES
// ==============================
use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use sqlx::{PgPool, FromRow, Row};
use tokio::time::{sleep, Duration};
use chrono::{Utc, Duration as ChronoDuration};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use anyhow::Result;
use aes_gcm::{
    Aes256Gcm,
    Key,
    Nonce,
    aead::{Aead, KeyInit}
};
use rand::{RngCore, thread_rng};
use base64::engine::general_purpose;
use base64::Engine;

use std::env;




#[derive(FromRow)]
struct User {
    id: i32,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterInput {
    email: String,
    password: String,
    confirm_password: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Serialize, FromRow)]
struct Email {
    id: i32,
    sender: String,
    subject: String,
    body: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(serde::Serialize, FromRow)]
struct Account {
    id: i32,
    email: String,
}

#[derive(Deserialize)]
struct LoginInput {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: i32,
    email: String,
    exp: usize,
}

#[post("/api/register")]
async fn register(
    pool: web::Data<PgPool>,
    data: web::Json<RegisterInput>,
) -> HttpResponse {
    log_auth("simple message");
    log_auth(format!("User registered: {}", data.email));
    if data.password != data.confirm_password {
        log_auth(&format!("Register failed (password mismatch): {}", data.email));
        return HttpResponse::BadRequest().json(
            serde_json::json!({ "message": "Passwords do not match" })
        );
    }
    log_auth(&format!("User registered successfully: {}", data.email));

    // 🔥 HASH PASSWORD
    let hashed = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            println!("Hash error: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({ "message": "Password hashing failed" })
            );
        }
    };

    let result = sqlx::query(
        "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id"
    )
    .bind(&data.email)
    .bind(&hashed) // ✅ FIXED
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let user_id: i32 = row.get("id");

            let claims = Claims {
                sub: user_id,
                email: data.email.clone(),
                exp: (Utc::now() + ChronoDuration::hours(24)).timestamp() as usize,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret("secret".as_ref()),
            ).unwrap();

            HttpResponse::Ok().json(
                serde_json::json!({ "token": token })
            )
        }

        Err(e) => {
            println!("DB ERROR: {:?}", e);

            if e.to_string().contains("duplicate key") {
                HttpResponse::BadRequest().json(
                    serde_json::json!({ "message": "User already exists" })
                )
            } else {
                HttpResponse::InternalServerError().json(
                    serde_json::json!({ "message": "Insert failed" })
                )
            }
        }
    }
}


#[post("/api/login")]
async fn login(
    pool: web::Data<PgPool>,
    data: web::Json<LoginInput>,
) -> HttpResponse {

    println!("Login attempt: {}", data.email);

    // ✅ HANDLE DB RESULT PROPERLY
    let user_result = sqlx::query_as::<_, User>(
        "SELECT id, email, password FROM users WHERE email = $1"
    )
    .bind(&data.email)
    .fetch_optional(pool.get_ref())
    .await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            println!("User not found");
            return HttpResponse::Unauthorized().json(MessageResponse {
                message: "Invalid credentials".to_string(),
            });
        }
        Err(e) => {
            println!("DB ERROR: {:?}", e);
            return HttpResponse::InternalServerError().json(MessageResponse {
                message: "Database error".to_string(),
            });
        }
    };

    // ✅ SAFE bcrypt check
    let valid = match verify(&data.password, &user.password) {
        Ok(v) => v,
        Err(e) => {
            println!("bcrypt verify error: {:?}", e);

            // 🔥 THIS IS YOUR CURRENT 500 ROOT CAUSE
            return HttpResponse::InternalServerError().json(MessageResponse {
                message: "Password verification failed".to_string(),
            });
        }
    };

    if !valid {
        return HttpResponse::Unauthorized().json(MessageResponse {
            message: "Invalid credentials".to_string(),
        });
    }

    // ✅ CREATE TOKEN
    let token = create_jwt(user.id, user.email.clone());

    HttpResponse::Ok().json(LoginResponse { token })
}



fn create_jwt(user_id: i32, email: String) -> String {
    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    )
    .unwrap()
}

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Email Import Running")
}


#[derive(serde::Deserialize, Default)]
pub struct EmailQuery {
    #[serde(default)]
    pub account_id: Option<i32>,

    #[serde(default)]
    pub before: Option<NaiveDateTime>, // 🔥 MUST BE STRING

    #[serde(default)]
    pub before_id: Option<i32>,
}


#[get("/api/emails")]
async fn get_emails(
    pool: web::Data<PgPool>,
    query: web::Query<EmailQuery>,
) -> impl Responder {

    let query = query.into_inner();

    let result = if let Some(before_time) = query.before {
        let before_id = query.before_id.unwrap_or(i32::MAX);

        if let Some(account_id) = query.account_id {
            // ✅ PAGINATION (SPECIFIC ACCOUNT)
            sqlx::query(
                r#"
                SELECT id, sender, subject, body_encrypted, body_iv, created_at
                FROM emails
                WHERE account_id = $1
                AND (
                    created_at < $2::timestamp
                    OR (created_at = $2::timestamp AND id < $3)
                )
                ORDER BY created_at DESC, id DESC
                LIMIT 50
                "#
            )
            .bind(account_id)
            .bind(before_time)
            .bind(before_id)
            .fetch_all(pool.get_ref())
            .await

        } else {
            // ✅ PAGINATION (ALL EMAILS)
            sqlx::query(
                r#"
                SELECT id, sender, subject, body_encrypted, body_iv, created_at
                FROM emails
                WHERE (
                    created_at < $1::timestamp
                    OR (created_at = $1::timestamp AND id < $2)
                )
                ORDER BY created_at DESC, id DESC
                LIMIT 50
                "#
            )
            .bind(before_time)
            .bind(before_id)
            .fetch_all(pool.get_ref())
            .await
        }

    } else {
        if let Some(account_id) = query.account_id {
            // ✅ INITIAL LOAD (SPECIFIC ACCOUNT)
            sqlx::query(
                r#"
                SELECT id, sender, subject, body_encrypted, body_iv, created_at
                FROM emails
                WHERE account_id = $1
                ORDER BY created_at DESC, id DESC
                LIMIT 50
                "#
            )
            .bind(account_id)
            .fetch_all(pool.get_ref())
            .await
        } else {
            // ✅ INITIAL LOAD (ALL EMAILS)
            sqlx::query(
                r#"
                SELECT id, sender, subject, body_encrypted, body_iv, created_at
                FROM emails
                ORDER BY created_at DESC, id DESC
                LIMIT 50
                "#
            )
            .fetch_all(pool.get_ref())
            .await
        }
    };

    match result {
        Ok(rows) => {
            let emails: Vec<_> = rows.into_iter().map(|row| {
                let id: i32 = row.try_get("id").unwrap();
                let sender: String = row.try_get("sender").unwrap();
                let subject: String = row.try_get("subject").unwrap();
                let created_at: chrono::NaiveDateTime =
                    row.try_get("created_at").unwrap();

                let iv: Option<String> = row.try_get("body_iv").ok();
                let enc: Option<String> = row.try_get("body_encrypted").ok();

                let body = if let (Some(iv), Some(enc)) = (iv, enc) {
                    decrypt(&iv, &enc)
                } else {
                    "".to_string()
                };

                serde_json::json!({
                    "id": id,
                    "sender": sender,
                    "subject": subject,
                    "body": body,
                    "created_at": created_at
                })
            }).collect();

            HttpResponse::Ok().json(emails)
        }

        Err(e) => {
            println!("❌ DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}



#[get("/api/accounts")]
async fn get_accounts(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, Account>(
        r#"
        SELECT id, email FROM email_accounts
        LIMIT 50
        "#
    )
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

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct UserResponse {
    id: i32,
    email: String,
}


#[get("/api/users")]
async fn get_users(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, UserResponse>(
        r#"
        SELECT id, email FROM users
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");

    let pool = PgPool::connect(&db_url)
        .await
        .expect("DB failed");

    let pool_clone = pool.clone();

    tokio::spawn(async move {
        loop {
            println!("Sync loop");

            if let Err(e) = gmail::sync_all(&pool_clone).await {
                println!("sync error {:?}", e);
            }

            sleep(Duration::from_secs(60)).await;
        }
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost")
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://127.0.0.1:5173")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
            ])
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(index)
            .service(register)
            .service(login)
            .service(send)
            .service(create_meeting)
            .service(get_meetings)
            .service(get_emails)
            .service(get_accounts)
            .service(get_messages)
            .service(get_users)
            .service(upload_file)
            .service(get_files)
            .route("/gmail/login", web::get().to(gmail_login))
            .route("/oauth/callback", web::get().to(oauth_callback))
            .route("/ws/chat", web::get().to(chat_ws))
            .route("/ws/call", web::get().to(call_ws))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}