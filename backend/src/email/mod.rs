use crate::prelude::*;

use reqwest::Client;
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::fs;
use futures::stream::{FuturesUnordered, StreamExt};
use once_cell::sync::Lazy;
use anyhow::Result;
use actix_web::{get, web, HttpResponse, Responder};
use tokio;

const MAX_EMAIL_CONCURRENCY: usize = 10; // 🔥 optimized
const BATCH_SIZE: usize = 50;


#[derive(Deserialize)]
struct EmailQuery {
    account_id: i32,
    last_seen: Option<String>,
}


// ============================
// GLOBAL HTTP CLIENT
// ============================

pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(20)
        .build()
        .unwrap()
});

// ============================
// MODELS
// ============================

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
}

// ============================
// LOAD GOOGLE SECRETS
// ============================

fn load_google_secrets() -> Value {
    let data = fs::read_to_string("client_secret.json")
        .expect("Failed to read client_secret.json");

    serde_json::from_str(&data).unwrap()
}

// ============================
// OAUTH CALLBACK
// ============================

pub async fn oauth_callback(
    pool: web::Data<PgPool>,
    query: web::Query<CallbackQuery>,
) -> impl Responder {

    let secrets = load_google_secrets();

    let client_id = secrets["web"]["client_id"].as_str().unwrap().to_string();
    let client_secret = secrets["web"]["client_secret"].as_str().unwrap().to_string();
    let redirect_uri = secrets["web"]["redirect_uris"][0].as_str().unwrap().to_string();

    // 🔁 exchange code → token
    let res: Value = HTTP_CLIENT
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", &query.code),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("redirect_uri", &redirect_uri),
            ("grant_type", &"authorization_code".to_string()),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let access_token = res["access_token"].as_str().unwrap_or("").to_string();
    let refresh_token = res["refresh_token"].as_str().unwrap_or("").to_string();
    let expires_in = res["expires_in"].as_i64().unwrap_or(3600);
    let expiry = chrono::Utc::now() + chrono::Duration::seconds(expires_in);

    // 🔍 get user email
    let user_info: Value = HTTP_CLIENT
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(&access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let email = user_info["email"].as_str().unwrap_or("").to_string();

    // 💾 save account
    let _ = sqlx::query(
        r#"
        INSERT INTO email_accounts
        (email, access_token, refresh_token, token_expiry, is_active)
        VALUES ($1,$2,$3,$4,true)
        ON CONFLICT (email)
        DO UPDATE SET
            access_token = EXCLUDED.access_token,
            token_expiry = EXCLUDED.token_expiry,
            refresh_token = COALESCE(
                NULLIF(EXCLUDED.refresh_token, ''),
                email_accounts.refresh_token
            )
        "#
    )
    .bind(&email)
    .bind(&access_token)
    .bind(&refresh_token)
    .bind(expiry)
    .execute(pool.get_ref())
    .await;

    println!("✅ Account connected is mod.rs code: {}", email);

    // 🔥 BACKGROUND SYNC
    let pool_clone = pool.get_ref().clone();
    tokio::spawn(async move {
        println!("🚀 Background sync started: yes);
        let _ = sync_all(&pool_clone).await;
    });

    // 🔁 redirect immediately
    HttpResponse::Found()
        .append_header(("Location", "http://localhost/emails"))
        .finish()
}

// ============================
// SYNC ALL (PARALLEL)
// ============================

pub async fn sync_all(pool: &PgPool) -> Result<()> {

    let rows = sqlx::query(
        "SELECT id, refresh_token, last_sync FROM email_accounts WHERE is_active = true"
    )
    .fetch_all(pool)
    .await?;

    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"].as_str().unwrap().to_string();
    let client_secret = secrets["web"]["client_secret"].as_str().unwrap().to_string();

    for r in rows {

        let pool = pool.clone();

        let account_id: i32 = r.get("id");
        let refresh_token: String = r.get("refresh_token");
        let last_sync: Option<i64> = r.try_get("last_sync").ok();

        let client_id = client_id.clone();
        let client_secret = client_secret.clone();

        tokio::spawn(async move {

            println!("📬 Syncing account: {}", account_id);

            let token = match refresh_access_token(
                &client_id,
                &client_secret,
                &refresh_token,
            ).await {
                Ok(t) => t,
                Err(_) => return,
            };

            let _ = sqlx::query(
                "UPDATE email_accounts SET access_token=$1 WHERE id=$2"
            )
            .bind(&token)
            .bind(account_id)
            .execute(&pool)
            .await;

            let _ = sync_account(&pool, account_id, &token, last_sync).await;
        });
    }

    Ok(())
}

// ============================
// FETCH IDS (LIMITED)
// ============================

async fn fetch_ids(token: &str, last_sync: Option<i64>) -> Result<Vec<String>> {

    let mut ids = Vec::new();
    let mut page_token: Option<String> = None;

    let mut total = 0;
    const MAX_TOTAL: usize = 100; // 🔥 fast UX

    let query = if let Some(ts) = last_sync {
        format!("&q=after:{}", ts - 3600)
    } else {
        "".to_string()
    };

    loop {
        let mut url = format!(
            "https://gmail.googleapis.com/gmail/v1/users/me/messages?maxResults=50{}",
            query
        );

        if let Some(ref t) = page_token {
            url.push_str(&format!("&pageToken={}", t));
        }

        let res: Value = HTTP_CLIENT
            .get(&url)
            .bearer_auth(token)
            .send()
            .await?
            .json()
            .await?;

        if let Some(messages) = res["messages"].as_array() {
            for m in messages {
                if total >= MAX_TOTAL { break; }

                if let Some(id) = m["id"].as_str() {
                    ids.push(id.to_string());
                    total += 1;
                }
            }
        }

        if total >= MAX_TOTAL { break; }

        page_token = res["nextPageToken"]
            .as_str()
            .map(|s| s.to_string());

        if page_token.is_none() { break; }
    }

    Ok(ids)
}

// ============================
// SYNC ACCOUNT
// ============================

async fn sync_account(
    pool: &PgPool,
    account_id: i32,
    token: &str,
    last_sync: Option<i64>,
) -> Result<()> {

    let ids = fetch_ids(token, last_sync).await?;

    let mut tasks = FuturesUnordered::new();

    for id in ids {
        let token = token.to_string();

        tasks.push(async move {
            fetch_email_detail(&token, &id).await
        });

        if tasks.len() >= MAX_EMAIL_CONCURRENCY {
            process_batch(pool, account_id, &mut tasks).await?;
        }
    }

    while !tasks.is_empty() {
        process_batch(pool, account_id, &mut tasks).await?;
    }

    sqlx::query("UPDATE email_accounts SET last_sync = $1 WHERE id = $2")
        .bind(chrono::Utc::now().timestamp())
        .bind(account_id)
        .execute(pool)
        .await?;

    Ok(())
}

// ============================
// BATCH INSERT
// ============================

async fn process_batch<F>(
    pool: &PgPool,
    account_id: i32,
    tasks: &mut FuturesUnordered<F>,
) -> Result<()>
where
    F: std::future::Future<Output = Result<(String, String, String, String, String)>>
{
    let mut batch = vec![];

    for _ in 0..BATCH_SIZE {
        if let Some(res) = tasks.next().await {
            if let Ok(v) = res {
                println!("📩 {} → {}", v.1, v.3);
                batch.push(v);
            }
        } else {
            break;
        }
    }

    if batch.is_empty() {
        return Ok(());
    }

    let mut query = String::from(
        "INSERT INTO emails(gmail_id, sender, receiver, subject, body, account_id) VALUES "
    );

    for (i, _) in batch.iter().enumerate() {
        let idx = i * 6;
        query.push_str(&format!(
            "(${}, ${}, ${}, ${}, ${}, ${}),",
            idx+1, idx+2, idx+3, idx+4, idx+5, idx+6
        ));
    }

    query.pop();
    query.push_str(" ON CONFLICT (account_id, gmail_id) DO NOTHING");

    let mut q = sqlx::query(&query);

    for (g,s,r,sub,b) in batch.iter() {
        q = q.bind(g).bind(s).bind(r).bind(sub).bind(b).bind(account_id);
    }

    q.execute(pool).await?;

    Ok(())
}

// ============================
// FETCH EMAIL DETAIL (FAST)
// ============================

async fn fetch_email_detail(
    token: &str,
    msg_id: &str,
) -> Result<(String, String, String, String, String)> {

    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=metadata&metadataHeaders=Subject&metadataHeaders=From&metadataHeaders=To",
        msg_id
    );

    let res: Value = HTTP_CLIENT
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .json()
        .await?;

    let headers = &res["payload"]["headers"];

    let mut subject = String::new();
    let mut sender = String::new();
    let mut receiver = String::new();

    if let Some(arr) = headers.as_array() {
        for h in arr {
            let name = h["name"].as_str().unwrap_or("");
            let value = h["value"].as_str().unwrap_or("");

            match name {
                "Subject" => subject = value.to_string(),
                "From" => sender = value.to_string(),
                "To" => receiver = value.to_string(),
                _ => {}
            }
        }
    }

    let snippet = res["snippet"].as_str().unwrap_or("").to_string();

    Ok((msg_id.to_string(), sender, receiver, subject, snippet))
}

// ============================
// REFRESH TOKEN
// ============================

async fn refresh_access_token(
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

    Ok(res["access_token"].as_str().unwrap_or("").to_string())
}