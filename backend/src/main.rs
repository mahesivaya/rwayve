// ==============================
// 🔹 INTERNAL MODULES (declare first)
// ==============================
mod ai;
mod cache;
mod call;
mod chat;
mod drive;
mod email;
mod external;
mod middleware;
mod models;
mod notes;
mod observability;
mod prelude;
mod routes;
mod scheduler;
pub mod security;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

// ==============================
// 🔹 USE INTERNAL MODULES
// ==============================
// use crate::middleware::logger::LoggerMiddleware;
// use crate::middleware::auth::AuthMiddleware;
// use crate::middleware::metrics::MetricsMiddleware;
// use crate::middleware::rate_limit::RateLimitMiddleware;

use crate::observability::devlog::init_devlog;
use crate::observability::tracing::init_tracing;
// 🚧 use crate::observability::tracing_root::AppRootSpanBuilder; // disabled

use crate::email::body_worker::run_body_worker;
use crate::email::sync::sync_all;

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
use tracing_actix_web::TracingLogger;

fn load_env_files() {
    // `cargo run` from backend/ loads backend/.env via dotenv().
    // Running from the repo root needs this explicit path. Docker Compose
    // injects backend/.env through env_file, so missing files are fine.
    dotenv().ok();
    dotenvy::from_filename("backend/.env").ok();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeRole {
    Api,
    EmailSyncWorker,
    EmailBodyWorker,
    All,
}

impl RuntimeRole {
    fn from_env() -> Self {
        match env::var("RWAYVE_ROLE").as_deref() {
            Ok("email-sync-worker") => Self::EmailSyncWorker,
            Ok("email-body-worker") => Self::EmailBodyWorker,
            Ok("all") => Self::All,
            _ => Self::Api,
        }
    }
}

fn db_max_connections(role: RuntimeRole) -> u32 {
    if let Ok(value) = env::var("DATABASE_MAX_CONNECTIONS") {
        if let Ok(parsed) = value.parse::<u32>() {
            return parsed;
        }

        warn!(
            value,
            "Invalid DATABASE_MAX_CONNECTIONS value; using role default"
        );
    }

    match role {
        RuntimeRole::Api | RuntimeRole::All => 10,
        RuntimeRole::EmailSyncWorker | RuntimeRole::EmailBodyWorker => 5,
    }
}

fn app_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // 🔥 GROUP API ROUTES
        .service(
            web::scope("/api")
                .configure(routes::routes)
                .configure(email::routes)
                .configure(chat::routes)
                .configure(scheduler::routes)
                .configure(drive::routes)
                .configure(notes::routes)
                .configure(ai::routes),
        )
        // 🔥 AUTH / GOOGLE
        .configure(email::public_routes)
        // 🔥 WEBSOCKETS
        .configure(chat::ws_routes)
        .configure(call::routes)
        // 🔥 STATIC FILES
        .service(Files::new("/uploads", "./uploads").show_files_listing());
}

async fn run_sync_worker(pool: PgPool) -> ! {
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
}

async fn ensure_schema(pool: &PgPool) {
    for statement in [
        r#"
        CREATE TABLE IF NOT EXISTS channels (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            visibility TEXT NOT NULL DEFAULT 'private',
            created_by INT REFERENCES users(id) ON DELETE CASCADE,
            created_at TIMESTAMP DEFAULT NOW()
        )
        "#,
        "ALTER TABLE channels ADD COLUMN IF NOT EXISTS visibility TEXT NOT NULL DEFAULT 'private'",
        r#"
        CREATE TABLE IF NOT EXISTS channel_members (
            channel_id INT REFERENCES channels(id) ON DELETE CASCADE,
            user_id INT REFERENCES users(id) ON DELETE CASCADE,
            role TEXT NOT NULL DEFAULT 'user',
            joined_at TIMESTAMP DEFAULT NOW(),
            PRIMARY KEY (channel_id, user_id)
        )
        "#,
        "ALTER TABLE channel_members ADD COLUMN IF NOT EXISTS role TEXT NOT NULL DEFAULT 'user'",
        "UPDATE channel_members cm SET role = 'admin' FROM channels c WHERE c.id = cm.channel_id AND c.created_by = cm.user_id",
        r#"
        CREATE TABLE IF NOT EXISTS channel_join_requests (
            channel_id INT REFERENCES channels(id) ON DELETE CASCADE,
            user_id INT REFERENCES users(id) ON DELETE CASCADE,
            status TEXT NOT NULL DEFAULT 'pending',
            requested_at TIMESTAMP DEFAULT NOW(),
            PRIMARY KEY (channel_id, user_id)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS channel_invites (
            id SERIAL PRIMARY KEY,
            channel_id INT REFERENCES channels(id) ON DELETE CASCADE,
            email TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            invited_by INT REFERENCES users(id) ON DELETE SET NULL,
            created_at TIMESTAMP DEFAULT NOW(),
            UNIQUE(channel_id, email)
        )
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS channel_messages (
            id SERIAL PRIMARY KEY,
            channel_id INT REFERENCES channels(id) ON DELETE CASCADE,
            sender_id INT REFERENCES users(id) ON DELETE CASCADE,
            content_encrypted TEXT,
            content_iv TEXT,
            created_at TIMESTAMP DEFAULT NOW()
        )
        "#,
        "CREATE INDEX IF NOT EXISTS idx_channel_members_user ON channel_members (user_id, channel_id)",
        "CREATE INDEX IF NOT EXISTS idx_channel_join_requests_channel ON channel_join_requests (channel_id, status)",
        "CREATE INDEX IF NOT EXISTS idx_channel_invites_channel ON channel_invites (channel_id, email)",
        "CREATE INDEX IF NOT EXISTS idx_channel_messages_channel_created ON channel_messages (channel_id, created_at DESC)",
    ] {
        if let Err(e) = sqlx::query(statement).execute(pool).await {
            error!("Failed to ensure chat channel schema: {:?}", e);
        }
    }

    if let Err(e) = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS email_attachments (
            id SERIAL PRIMARY KEY,
            email_id INTEGER NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
            account_id INTEGER NOT NULL REFERENCES email_accounts(id) ON DELETE CASCADE,
            gmail_id TEXT NOT NULL,
            attachment_id TEXT NOT NULL,
            filename TEXT NOT NULL,
            mime_type TEXT,
            size BIGINT DEFAULT 0,
            created_at TIMESTAMP DEFAULT NOW(),
            UNIQUE(email_id, attachment_id)
        )
        "#,
    )
    .execute(pool)
    .await
    {
        error!("Failed to ensure email_attachments table: {:?}", e);
    }

    if let Err(e) = sqlx::query(
        "ALTER TABLE emails ADD COLUMN IF NOT EXISTS attachments_checked BOOLEAN DEFAULT FALSE",
    )
    .execute(pool)
    .await
    {
        error!(
            "Failed to ensure emails.attachments_checked column: {:?}",
            e
        );
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracing();
    init_devlog();
    load_env_files();
    info!("Server starting...");
    tracing::info!("Server starting...");
    let role = RuntimeRole::from_env();
    info!(?role, "Runtime role selected");

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| panic!("DATABASE_URL missing"));
    let max_db_connections = db_max_connections(role);
    info!(max_db_connections, "Database pool size selected");
    // Log the first failure verbosely; subsequent identical failures get a
    // compact dot-counter so dev.log doesn't fill with the same line.
    let pool = {
        let mut attempts: u32 = 0;
        loop {
            match PgPoolOptions::new()
                .max_connections(max_db_connections)
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

    ensure_schema(&pool).await;

    match role {
        RuntimeRole::EmailSyncWorker => run_sync_worker(pool).await,
        RuntimeRole::EmailBodyWorker => run_body_worker(pool).await,
        RuntimeRole::All => {
            let sync_pool = pool.clone();
            tokio::spawn(async move {
                run_sync_worker(sync_pool).await;
            });
            let body_pool = pool.clone();
            tokio::spawn(async move {
                run_body_worker(body_pool).await;
            });
        }
        RuntimeRole::Api => {}
    }

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
            .expose_headers(vec![actix_web::http::header::HeaderName::from_static(
                "x-has-more",
            )])
            .supports_credentials();

        App::new()
            .wrap(TracingLogger::default()) // 🚧
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
