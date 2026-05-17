use moka::future::Cache as MokaCache;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::env;
use std::time::Duration;
use tracing::{instrument, warn};

const DEFAULT_LOCAL_JSON_CACHE_TTL_SECS: u64 = 60;
const DEFAULT_LOCAL_JSON_CACHE_MAX_CAPACITY: u64 = 10_000;

#[derive(Clone)]
pub struct Cache {
    conn: MultiplexedConnection,
    local_json: MokaCache<String, String>,
}

impl Cache {
    #[instrument(target = "cache")]
    pub async fn connect() -> Result<Self, redis::RedisError> {
        let url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".to_string());
        let client = redis::Client::open(url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self {
            conn,
            local_json: local_json_cache(),
        })
    }

    #[instrument(target = "cache", skip(self), fields(key))]
    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        if let Some(raw) = self.local_json.get(key).await {
            return serde_json::from_str(&raw).ok();
        }

        let mut conn = self.conn.clone();
        let raw: Option<String> = conn.get(key).await.ok()?;
        let raw = raw?;
        self.local_json.insert(key.to_string(), raw.clone()).await;
        serde_json::from_str(&raw).ok()
    }

    #[instrument(target = "cache", skip(self, value), fields(key, ttl_secs))]
    pub async fn set_json_with_ttl<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) {
        let raw = match serde_json::to_string(value) {
            Ok(s) => s,
            Err(e) => {
                warn!(target: "cache", key, error = ?e, "cache serialize failed");
                return;
            }
        };

        self.local_json.insert(key.to_string(), raw.clone()).await;

        let mut conn = self.conn.clone();
        let res: redis::RedisResult<()> = conn.set_ex(key, raw, ttl_secs).await;
        if let Err(e) = res {
            warn!(target: "cache", key, error = ?e, "redis SETEX failed");
        }
    }

    #[instrument(target = "cache", skip(self), fields(key))]
    pub async fn del(&self, key: &str) {
        self.local_json.invalidate(key).await;
        let mut conn = self.conn.clone();
        let _: redis::RedisResult<()> = conn.del(key).await;
    }

    /// Round-trips a `PING` to confirm Redis is reachable. Used by the
    /// readiness probe; never panics — a transport error just means "down".
    #[instrument(target = "cache", skip(self))]
    pub async fn ping(&self) -> bool {
        let mut conn = self.conn.clone();
        let res: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut conn).await;
        res.is_ok()
    }

    #[instrument(target = "cache", skip(self), fields(key, ttl_secs))]
    pub async fn increment_with_ttl(&self, key: &str, ttl_secs: u64) -> redis::RedisResult<i64> {
        let mut conn = self.conn.clone();
        let count: i64 = conn.incr(key, 1).await?;

        if count == 1 {
            let _: bool = conn.expire(key, ttl_secs as i64).await?;
        }

        Ok(count)
    }
}

fn local_json_cache() -> MokaCache<String, String> {
    let ttl_secs = env::var("LOCAL_JSON_CACHE_TTL_SECS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_LOCAL_JSON_CACHE_TTL_SECS);
    let max_capacity = env::var("LOCAL_JSON_CACHE_MAX_CAPACITY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_LOCAL_JSON_CACHE_MAX_CAPACITY);

    MokaCache::builder()
        .max_capacity(max_capacity)
        .time_to_live(Duration::from_secs(ttl_secs))
        .build()
}

pub fn chat_history_key(a: i32, b: i32) -> String {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    format!("chat:msgs:{}:{}", lo, hi)
}
