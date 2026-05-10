use crate::prelude::*;
use tracing::{instrument, warn};

pub fn load_google_secrets() -> serde_json::Value {
    // Allow tests (and self-hosted setups with the secret elsewhere) to
    // override the path. Defaults to the production location.
    let path = std::env::var("GOOGLE_CLIENT_SECRET_PATH")
        .unwrap_or_else(|_| "client_secret.json".to_string());
    let data = fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));

    serde_json::from_str(&data).unwrap()
}

#[instrument(target = "gmail", skip_all)]
pub async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<String> {
    let res: Value = HTTP_CLIENT
        .post(crate::external::google_token_url())
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await?
        .json()
        .await?;

    if let Some(err) = res.get("error") {
        warn!(target: "gmail", error = %err, "google token refresh returned error");
        return Err(anyhow::anyhow!("Token refresh failed"));
    }

    Ok(res["access_token"].as_str().unwrap_or("").to_string())
}

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(20)
        .build()
        .unwrap()
});
