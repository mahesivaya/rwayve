use crate::email::sync::sync_all;
use sqlx::PgPool;
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};

pub async fn run_sync_worker(pool: PgPool) -> ! {
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
