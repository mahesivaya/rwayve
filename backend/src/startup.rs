use crate::ai;
use crate::call;
use crate::chat;
use crate::drive;
use crate::email;
use crate::notes;
use crate::routes;
use crate::scheduler;
use actix_web::web;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{info, instrument, warn};

#[instrument(target = "startup", skip(db_url), fields(max_conns))]
#[allow(dead_code)]
pub async fn establish_db_connection(db_url: &str, max_conns: u32) -> PgPool {
    let mut attempts: u32 = 0;
    loop {
        match PgPoolOptions::new()
            .max_connections(max_conns)
            .connect(db_url)
            .await
        {
            Ok(pool) => {
                if attempts > 0 {
                    info!("Connected to Postgres after {} retries", attempts);
                } else {
                    info!("Connected to Postgres");
                }
                return pool;
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
}

#[instrument(target = "startup", skip(pool))]
pub async fn ensure_email_schema(pool: &PgPool) {
    if let Err(e) =
        sqlx::query("ALTER TABLE emails ADD COLUMN IF NOT EXISTS is_read BOOLEAN DEFAULT TRUE")
            .execute(pool)
            .await
    {
        warn!(error = ?e, "email schema compatibility check failed");
    }
}

#[instrument(target = "startup", skip(cfg))]
#[allow(dead_code)]
pub fn configure_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(routes::routes)
            .configure(email::routes)
            .configure(chat::routes)
            .configure(scheduler::routes)
            .configure(drive::routes)
            .configure(notes::routes)
            .configure(ai::routes),
    )
    .configure(email::public_routes)
    .configure(chat::ws_routes)
    .configure(call::routes);
}
