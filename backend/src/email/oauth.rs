use crate::prelude::*;

pub fn load_google_secrets() -> serde_json::Value {
    let data = fs::read_to_string("client_secret.json")
        .unwrap_or_else(|e| panic!("Failed to read client_secret.json: {e}"));

    serde_json::from_str(&data).unwrap()
}

pub async fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<String> {
    let res: Value = HTTP_CLIENT
        .post("https://oauth2.googleapis.com/token")
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

    if res.get("error").is_some() {
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
