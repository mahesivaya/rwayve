// ==============================
// 🔹 INTERNAL MODULES (declare first)
// ==============================
mod call;
mod chat;
mod drive;
mod email;
mod logging;
mod models;
mod prelude;
mod routes;
mod scheduler;
pub mod security;
mod middleware;

// ==============================
// 🔹 USE INTERNAL MODULES
// ==============================
// use crate::middleware::logger::LoggerMiddleware;
// use crate::middleware::auth::AuthMiddleware;
// use crate::middleware::metrics::MetricsMiddleware;
// use crate::middleware::rate_limit::RateLimitMiddleware;

use crate::logging::logger::init_logger;

use crate::chat::handler::{chat_ws, get_messages};

use crate::drive::handler::{get_files, upload_file};

use crate::scheduler::handler::{create_meeting, delete_meeting, get_meetings, update_meeting};

use crate::call::handler::call_ws;

use crate::email::body_worker::start_body_worker;
use crate::email::handler::{
    get_email_body, get_me, gmail_login,
    oauth_callback, save_public_key, send,
    get_email_by_id
};
use crate::email::sync::sync_all;

use crate::routes::account::get_accounts;
use crate::routes::auth::{login, register};
use crate::routes::email::get_emails;
use crate::routes::user::{get_all_users, get_user_by_email};

// ==============================
// 🔹 EXTERNAL CRATES
// ==============================
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpServer, web};
pub use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sqlx::PgPool;
use tokio::time::{Duration, sleep};

use dotenvy::dotenv;
use std::env;

fn app_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // 🔥 GROUP API ROUTES
        .service(
            web::scope("/api")
                .service(register)
                .service(login)
                .service(get_emails)
                .service(get_email_body)
                .service(get_email_by_id)
                .service(get_accounts)
                .service(get_messages)
                .service(get_user_by_email)
                .service(get_all_users)
                .service(create_meeting)
                .service(get_meetings)
                .service(update_meeting)
                .service(delete_meeting)
                .service(upload_file)
                .service(get_files)
                .service(send)
                .service(get_me)
                .service(save_public_key)
        )
        // 🔥 AUTH / GOOGLE
        .route("/gmail/login", web::get().to(gmail_login))
        .route("/oauth/callback", web::get().to(oauth_callback))
        // 🔥 WEBSOCKETS
        .route("/ws/chat", web::get().to(chat_ws))
        .route("/ws/call", web::get().to(call_ws))
        // 🔥 STATIC FILES
        .service(Files::new("/uploads", "./uploads").show_files_listing());
}

fn start_sync_worker(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            println!("Sync loop");
            if let Err(e) = sync_all(&pool).await {
                println!("sync error {:?}", e);
            }
            sleep(Duration::from_secs(30)).await;
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Server starting...");
    init_logger();
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let pool = loop {
        match PgPool::connect(&db_url).await {
            Ok(pool) => {
                println!("✅ Connected to DB");
                break pool;
            }
            Err(e) => {
                println!("⏳ Waiting for DB... {:?}", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    };
    start_sync_worker(pool.clone());
    start_body_worker(pool.clone());

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
            // .wrap(LoggerMiddleware)        // logging
            // .wrap(MetricsMiddleware)       // performance
            // .wrap(RateLimitMiddleware)     // protection
            // .wrap(AuthMiddleware)          // security
            .app_data(web::Data::new(pool.clone()))
            .configure(app_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
