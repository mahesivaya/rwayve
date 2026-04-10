// Environment
use std::env;

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
mod handlers;
mod services;
mod workers;
mod routes;


// ==============================
// 🔹 USE INTERNAL MODULES
// ==============================
use crate::prelude::*;
use crate::logging::logger::init_logger;
use crate::chat::{chat_ws, get_messages};
use crate::gmail::oauth_callback;
use crate::drive::{upload_file, get_files};
use crate::scheduler::{create_meeting, get_meetings};
use crate::call::call::call_ws;
use crate::security::encryption::decrypt;
use crate::handlers::register::register;
use crate::handlers::login::login;
use crate::handlers::accounts::get_accounts;
use crate::handlers::emails::get_emails;
use crate::handlers::users::get_users;
use crate::handlers::send::send;
use crate::handlers::gmail::gmail_login;


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
            .service(get_accounts)
            .service(register)
            .service(login)
            .service(send)
            .service(create_meeting)
            .service(get_meetings)
            .service(get_emails)
            .service(get_messages)
            .service(get_users)
            .service(upload_file)
            .service(get_files)
            .service(gmail_login)
            .route("/oauth/callback", web::get().to(oauth_callback))
            .route("/ws/chat", web::get().to(chat_ws))
            .route("/ws/call", web::get().to(call_ws))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}