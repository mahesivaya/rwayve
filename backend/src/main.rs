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
use crate::observability::devlog::init_devlog;
use crate::observability::tracing::init_tracing;
// 🚧 use crate::observability::tracing_root::AppRootSpanBuilder; // disabled

use crate::email::body_worker::run_body_worker;
use crate::email::sync::sync_all;
use crate::middleware::rate_limit::RateLimitMiddleware;

// ==============================
// 🔹 EXTERNAL CRATES
// ==============================
use actix_cors::Cors;
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
        .configure(call::routes);
    // NOTE: uploaded files are no longer served statically from /uploads.
    // They are delivered via the authenticated, ownership-checked route
    // GET /api/files/{id}/download (see drive::handler::download_file).
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracing();
    init_devlog();
    load_env_files();
    crate::security::jwt::jwt_secret();
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

    // Listen port: PORT env var, falling back to 8080 when unset/invalid.
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8080);
    info!(port, "Listen port selected");

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
            .wrap(RateLimitMiddleware)
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(redis_cache.clone()))
            .configure(app_routes)
    })
    .bind(("0.0.0.0", port))?;

    info!("Server started on :{port}");

    let res = server.run().await;
    info!("Server shutdown complete");
    res
}
