use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Deserialize)]
struct TokenResp {
    access_token: String,
}

#[derive(Deserialize)]
struct MeetingResp {
    join_url: String,
}

async fn fetch_access_token() -> Result<String, String> {
    let account_id =
        env::var("ZOOM_ACCOUNT_ID").map_err(|_| "ZOOM_ACCOUNT_ID not set".to_string())?;
    let client_id = env::var("ZOOM_CLIENT_ID").map_err(|_| "ZOOM_CLIENT_ID not set".to_string())?;
    let client_secret =
        env::var("ZOOM_CLIENT_SECRET").map_err(|_| "ZOOM_CLIENT_SECRET not set".to_string())?;

    let basic = STANDARD.encode(format!("{}:{}", client_id, client_secret));

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {:?}", e))?;

    let res = client
        .post("https://zoom.us/oauth/token")
        .header("Authorization", format!("Basic {}", basic))
        .form(&[
            ("grant_type", "account_credentials"),
            ("account_id", account_id.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("Zoom token request failed: {:?}", e))?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Zoom token error: {}", text));
    }

    let tok: TokenResp = res
        .json()
        .await
        .map_err(|e| format!("Zoom token parse error: {:?}", e))?;
    Ok(tok.access_token)
}

pub async fn create_zoom_meeting(
    topic: &str,
    start_utc: DateTime<Utc>,
    duration_min: i64,
) -> Result<String, String> {
    let token = fetch_access_token().await?;

    let body = json!({
        "topic": topic,
        "type": 2,
        "start_time": start_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "duration": duration_min,
        "timezone": "UTC",
        "settings": {
            "join_before_host": true,
            "approval_type": 2,
            "waiting_room": false
        }
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {:?}", e))?;

    let res = client
        .post("https://api.zoom.us/v2/users/me/meetings")
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Zoom create request failed: {:?}", e))?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Zoom create error: {}", text));
    }

    let m: MeetingResp = res
        .json()
        .await
        .map_err(|e| format!("Zoom meeting parse error: {:?}", e))?;
    Ok(m.join_url)
}
