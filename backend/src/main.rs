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
    // Running from the repo root needs explicit paths. Docker Compose injects
    // environment files through env_file, so missing local files are fine.
    dotenv().ok();

    if let Ok(env_file) = env::var("ENV_FILE") {
        dotenvy::from_filename_override(env_file).ok();
    }

    let app_env = env::var("RWAYVE_ENV")
        .or_else(|_| env::var("ENV"))
        .unwrap_or_else(|_| "development".to_string());
    dotenvy::from_filename_override(format!(".env.{app_env}")).ok();
    dotenvy::from_filename_override(format!("backend/.env.{app_env}")).ok();
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

/// HTTP listen port: the `PORT` env var, falling back to 8080 when unset or
/// not a valid `u16`.
fn listen_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8080)
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

    let port = listen_port();
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

#[cfg(test)]
mod runtime_tests {
    // Named `runtime_tests` (not `tests`) to avoid colliding with the
    // `#[cfg(test)] mod tests;` directory module declared above.
    use super::*;

    #[test]
    #[serial_test::serial]
    fn runtime_role_defaults_to_api() {
        unsafe { env::remove_var("RWAYVE_ROLE") };
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::Api);
    }

    #[test]
    #[serial_test::serial]
    fn runtime_role_parses_known_values() {
        unsafe { env::set_var("RWAYVE_ROLE", "email-sync-worker") };
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::EmailSyncWorker);
        unsafe { env::set_var("RWAYVE_ROLE", "email-body-worker") };
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::EmailBodyWorker);
        unsafe { env::set_var("RWAYVE_ROLE", "all") };
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::All);
        unsafe { env::set_var("RWAYVE_ROLE", "nonsense") };
        assert_eq!(RuntimeRole::from_env(), RuntimeRole::Api);
        unsafe { env::remove_var("RWAYVE_ROLE") };
    }

    #[test]
    #[serial_test::serial]
    fn db_max_connections_uses_role_defaults() {
        unsafe { env::remove_var("DATABASE_MAX_CONNECTIONS") };
        assert_eq!(db_max_connections(RuntimeRole::Api), 10);
        assert_eq!(db_max_connections(RuntimeRole::All), 10);
        assert_eq!(db_max_connections(RuntimeRole::EmailSyncWorker), 5);
        assert_eq!(db_max_connections(RuntimeRole::EmailBodyWorker), 5);
    }

    #[test]
    #[serial_test::serial]
    fn db_max_connections_honors_valid_override() {
        unsafe { env::set_var("DATABASE_MAX_CONNECTIONS", "42") };
        assert_eq!(db_max_connections(RuntimeRole::Api), 42);
        assert_eq!(db_max_connections(RuntimeRole::EmailSyncWorker), 42);
        unsafe { env::remove_var("DATABASE_MAX_CONNECTIONS") };
    }

    #[test]
    #[serial_test::serial]
    fn db_max_connections_ignores_invalid_override() {
        unsafe { env::set_var("DATABASE_MAX_CONNECTIONS", "not-a-number") };
        assert_eq!(db_max_connections(RuntimeRole::Api), 10);
        unsafe { env::remove_var("DATABASE_MAX_CONNECTIONS") };
    }

    #[test]
    #[serial_test::serial]
    fn listen_port_defaults_when_unset() {
        unsafe { env::remove_var("PORT") };
        assert_eq!(listen_port(), 8080);
    }

    #[test]
    #[serial_test::serial]
    fn listen_port_parses_and_falls_back() {
        unsafe { env::set_var("PORT", "9090") };
        assert_eq!(listen_port(), 9090);
        unsafe { env::set_var("PORT", "garbage") };
        assert_eq!(listen_port(), 8080);
        unsafe { env::remove_var("PORT") };
    }
}
