// ==============================
// 🔹 INTERNAL MODULES (declare first)
// ==============================
mod ai;
mod cache;
mod call;
mod chat;
mod drive;
mod email;
mod middleware;
mod models;
mod notes;
mod observability;
mod prelude;
mod routes;
mod scheduler;
pub mod security;

// ==============================
// 🔹 USE INTERNAL MODULES
// ==============================
// use crate::middleware::logger::LoggerMiddleware;
// use crate::middleware::auth::AuthMiddleware;
// use crate::middleware::metrics::MetricsMiddleware;
// use crate::middleware::rate_limit::RateLimitMiddleware;

use crate::observability::devlog::init_devlog;
use crate::observability::logger::init_logger;
// 🚧 use crate::observability::tracing_root::AppRootSpanBuilder; // disabled

use crate::chat::handler::{chat_ws, get_messages};

use crate::drive::handler::{get_files, upload_file};

use crate::scheduler::handler::{create_meeting, delete_meeting, get_meetings, update_meeting};

use crate::call::handler::call_ws;

use crate::notes::handler::{create_note, delete_note, list_notes, update_note};

use crate::ai::handler::ai_chat;

use crate::email::body_worker::start_body_worker;
use crate::email::handler::{
    get_email_body, get_email_by_id, get_me, gmail_login, oauth_callback, save_public_key, send,
};
use crate::email::sync::sync_all;

use crate::routes::account::{delete_account, get_accounts};
use crate::routes::auth::{forgot_password, login, register, reset_password};
use crate::routes::email::get_emails;
use crate::routes::user::{
    change_password, get_all_users, get_profile, get_user_by_email, update_profile,
};

// ==============================
// 🔹 EXTERNAL CRATES
// ==============================
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpServer, web};
pub use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::time::{Duration, sleep};

use dotenvy::dotenv;
use std::env;
use tracing::{error, info, warn};
// 🚧 use tracing_actix_web::TracingLogger; // disabled

fn app_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // 🔥 GROUP API ROUTES
        .service(
            web::scope("/api")
                .service(register)
                .service(login)
                .service(forgot_password)
                .service(reset_password)
                .service(change_password)
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
                .service(list_notes)
                .service(create_note)
                .service(update_note)
                .service(delete_note)
                .service(get_profile)
                .service(update_profile)
                .service(delete_account)
                .service(ai_chat),
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
        let mut interval = Duration::from_secs(30);
        info!("Sync worker started");

        loop {
            match sync_all(&pool).await {
                Ok(_) => {
                    info!("Sync cycle success");
                    interval = Duration::from_secs(30);
                }
                Err(e) => {
                    error!("Sync cycle failed: {:?}", e);
                    interval = std::cmp::min(interval * 2, Duration::from_secs(300));
                    warn!("Sync backoff: {:?}", interval);
                }
            }

            sleep(interval).await;
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();
    init_devlog();
    dotenv().ok();
    info!("Server starting...");

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| panic!("DATABASE_URL missing"));
    // Log the first failure verbosely; subsequent identical failures get a
    // compact dot-counter so dev.log doesn't fill with the same line.
    let pool = {
        let mut attempts: u32 = 0;
        loop {
            match PgPoolOptions::new()
                .max_connections(10)
                .connect(&db_url)
                .await
            {
                Ok(pool) => {
                    if attempts > 0 {
                        info!("Connected to Postgres after {} retries", attempts);
                    } else {
                        info!("Connected to Postgres");
                    }
                    break pool;
                }
                Err(e) => {
                    if attempts == 0 {
                        warn!("Postgres unavailable, retrying... ({e:?})");
                    } else if attempts.is_power_of_two() {
                        warn!("Postgres still unavailable after {} retries", attempts);
                    }
                    attempts += 1;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    };

    start_sync_worker(pool.clone());
    start_body_worker(pool.clone());

    let redis_cache = match crate::cache::Cache::connect().await {
        Ok(c) => {
            info!("Connected to Redis");
            Some(c)
        }
        Err(e) => {
            warn!("Redis unavailable, caching disabled ({e:?})");
            None
        }
    };

    let frontend_url = env::var("FRONTEND_URL").unwrap_or_else(|_| panic!("FRONTEND_URL missing"));

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&frontend_url)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::HeaderName::from_static("x-request-id"),
            ])
            .supports_credentials();

        App::new()
            // 🚧 .wrap(TracingLogger::<AppRootSpanBuilder>::new()) // disabled
            .wrap(cors)
            // .wrap(LoggerMiddleware)        // observability
            // .wrap(MetricsMiddleware)       // performance
            // .wrap(RateLimitMiddleware)     // protection
            // .wrap(AuthMiddleware)          // security
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(redis_cache.clone()))
            .configure(app_routes)
    })
    .bind(("0.0.0.0", 8080))?;

    info!("Server started on :8080");

    let res = server.run().await;
    info!("Server shutdown complete");
    res
}
