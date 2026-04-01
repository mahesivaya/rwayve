use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::env;
use tokio::time::{sleep, Duration};
use crate::gmail::{gmail_login, oauth_callback, send};
use crate::chat::chat_ws;
mod chat;
mod gmail;
mod scheduler;
use crate::scheduler::{create_meeting, get_meetings};

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
    if data.password != data.confirm_password {
        return HttpResponse::BadRequest().json(MessageResponse {
            message: "Passwords do not match".to_string(),
        });
    }

    let hashed = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(MessageResponse {
                message: "Failed to hash password".to_string(),
            });
        }
    };

    let result = sqlx::query(
        "INSERT INTO users (email, password) VALUES ($1, $2)"
    )
    .bind(&data.email)
    .bind(&hashed)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(MessageResponse {
            message: "User created".to_string(),
        }),
        Err(e) => {
            println!("register db error: {:?}", e);
            HttpResponse::BadRequest().json(MessageResponse {
                message: "User exists or insert failed".to_string(),
            })
        }
    }
}


#[post("/api/login")]
async fn login(
    pool: web::Data<PgPool>,
    data: web::Json<LoginInput>,
) -> HttpResponse {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password FROM users WHERE email = $1"
    )
    .bind(&data.email)
    .fetch_optional(pool.get_ref())
    .await;

    if let Ok(Some(user)) = user {
        match verify(&data.password, &user.password) {
            Ok(true) => {
                let token = create_jwt(user.id, user.email.clone());
                return HttpResponse::Ok().json(LoginResponse { token });
            }
            Ok(false) => {}
            Err(e) => {
                println!("bcrypt verify error: {:?}", e);
                return HttpResponse::InternalServerError().json(MessageResponse {
                    message: "Login failed".to_string(),
                });
            }
        }
    }

    HttpResponse::Unauthorized().json(MessageResponse {
        message: "Invalid credentials".to_string(),
    })
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

async fn get_emails(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, Email>(
        r#"
        SELECT id, sender, subject, body, created_at
        FROM emails
        ORDER BY created_at DESC
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


#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
            .route("/gmail/login", web::get().to(gmail_login))
            .route("/oauth/callback", web::get().to(oauth_callback))
            .route("/emails", web::get().to(get_emails))
            .route("/accounts", web::get().to(get_accounts))
            .route("/ws/chat", web::get().to(chat_ws))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}