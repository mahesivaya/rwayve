use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::env;

#[derive(Clone)]
pub struct Cache {
    conn: MultiplexedConnection,
}

impl Cache {
    pub async fn connect() -> Result<Self, redis::RedisError> {
        let url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".to_string());
        let client = redis::Client::open(url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self { conn })
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn.get(key).await.ok()?;
        let raw = raw?;
        serde_json::from_str(&raw).ok()
    }

    pub async fn set_json_with_ttl<T: Serialize>(&self, key: &str, value: &T, ttl_secs: u64) {
        let raw = match serde_json::to_string(value) {
            Ok(s) => s,
            Err(e) => {
                println!("⚠️ cache serialize error: {:?}", e);
                return;
            }
        };
        let mut conn = self.conn.clone();
        let _: redis::RedisResult<()> = conn.set_ex(key, raw, ttl_secs).await;
    }

    pub async fn del(&self, key: &str) {
        let mut conn = self.conn.clone();
        let _: redis::RedisResult<()> = conn.del(key).await;
    }
}

pub fn chat_history_key(a: i32, b: i32) -> String {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    format!("chat:msgs:{}:{}", lo, hi)
}
