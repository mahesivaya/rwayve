use crate::prelude::*;

use crate::email::utils::extract_body;
use crate::email::oauth::{load_google_secrets,refresh_access_token,HTTP_CLIENT};
use crate::security::encryption::encrypt;

use serde_json::Value;

pub async fn fetch_email_detail(
    token: &str,
    msg_id: &str,
) -> Result<(String, String, String, String, String)> {

    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=full",
        msg_id
    );

    let res: Value = HTTP_CLIENT
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .json()
        .await?;

    let payload = &res["payload"];

    // ✅ ONLY THIS (no override later)
    let (sender, receiver, subject) = extract_headers(&res);

    // optional snippet fallback
    let _snippet = res["snippet"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let body = extract_body(payload).unwrap_or_else(|| {
        res["snippet"].as_str().unwrap_or("").to_string()
    });
    println!("📧 {} | {} | {}", sender, receiver, subject);
    Ok((
        msg_id.to_string(),
        sender,
        receiver,
        subject,
        body,
    ))
}


pub async fn sync_all(pool: &PgPool) -> Result<()> {

    let rows = sqlx::query(
        "SELECT id, access_token, refresh_token, last_sync FROM email_accounts WHERE access_token IS NOT NULL"
    )
    .fetch_all(pool)
    .await?;

    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"].as_str().unwrap();
    let client_secret = secrets["web"]["client_secret"].as_str().unwrap();

    let mut handles = vec![];

    for r in rows {

        let pool = pool.clone();

        let account_id: i32 = r.get("id");
        let refresh_token: String = r.get("refresh_token");
        let last_sync: Option<i64> = r.try_get("last_sync").ok();

        let client_id = client_id.to_string();
        let client_secret = client_secret.to_string();

        let handle = tokio::spawn(async move {

            // 🔁 refresh token
            let token = match refresh_access_token(
                &client_id,
                &client_secret,
                &refresh_token,
            ).await {
                Ok(t) => t,
                Err(_e) => {
                    return;
                }
            };

            // save token
            let _ = sqlx::query(
                "UPDATE email_accounts SET access_token=$1 WHERE id=$2"
            )
            .bind(&token)
            .bind(account_id)
            .execute(&pool)
            .await;

            // sync
            if let Err(_e) = sync_account(&pool, account_id, &token, last_sync).await {
                println!("Sync error {}: {}", account_id, _e);
            }
        });

        handles.push(handle);
    }

    for h in handles {
        let _ = h.await;
    }

    Ok(())
}

pub async fn fetch_ids(
    token: &str,
    last_sync: Option<i64>,
) -> Result<Vec<String>> {

    let mut ids = Vec::new();
    let mut page_token: Option<String> = None;

    let query = if let Some(ts) = last_sync {
        // subtract 1 hour buffer (important)
        let safe_ts = ts - 3600;
        format!("&q=after:{}", safe_ts)
    } else {
        "".to_string()
    };

    loop {
        let mut url = format!(
            "https://gmail.googleapis.com/gmail/v1/users/me/messages?maxResults=100{}",
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
                if let Some(id) = m["id"].as_str() {
                    ids.push(id.to_string());
                }
            }
        }

        page_token = res["nextPageToken"]
            .as_str()
            .map(|s| s.to_string());

        if page_token.is_none() {
            break;
        }
    }
    Ok(ids)
}

// Extract Headers inside deep dive.

pub fn extract_headers(res: &Value) -> (String, String, String) {
    let mut sender: Option<String> = None;
    let mut receiver: Option<String> = None;
    let mut subject: Option<String> = None;

    // 🔁 recursive function to walk through payload
    fn walk_parts(
        node: &Value,
        sender: &mut Option<String>,
        receiver: &mut Option<String>,
        subject: &mut Option<String>,
    ) {
        // 1. Check headers at current level
        if let Some(headers) = node["headers"].as_array() {
            for h in headers {
                let name = h["name"].as_str().unwrap_or("");
                let value = h["value"].as_str().unwrap_or("").to_string();

                match name {
                    "From" => {
                        if sender.is_none() && !value.is_empty() {
                            *sender = Some(value);
                        }
                    }
                    "To" => {
                        if receiver.is_none() && !value.is_empty() {
                            *receiver = Some(value);
                        }
                    }
                    "Subject" => {
                        if subject.is_none() && !value.is_empty() {
                            *subject = Some(value);
                        }
                    }
                    _ => {}
                }
            }
        }

        // 2. Traverse deeper if parts exist
        if let Some(parts) = node["parts"].as_array() {
            for part in parts {
                walk_parts(part, sender, receiver, subject);
            }
        }
    }

    // 🚀 Start from payload root
    walk_parts(&res["payload"], &mut sender, &mut receiver, &mut subject);

    // 🛡️ Fallback defaults
    let sender = sender.unwrap_or_else(|| "Unknown".to_string());
    let receiver = receiver.unwrap_or_else(|| "Unknown".to_string());
    let subject = subject.unwrap_or_else(|| "(No Subject)".to_string());

    (sender, receiver, subject)
}

//////////////////////////////////////////////////
// SYNC ACCOUNT
//////////////////////////////////////////////////

pub async fn sync_account(
    pool: &PgPool,
    account_id: i32,
    token: &str,
    last_sync: Option<i64>,
) -> anyhow::Result<()> {

    let ids = fetch_ids(token, last_sync).await?;

    let mut tasks = FuturesUnordered::new();

    for id in ids {
        let token = token.to_string();

        // ✅ ONLY call fetch (no parsing here)
        tasks.push(async move {
            fetch_email_detail(&token, &id).await
        });

        // ✅ Process batch when limit reached
        if tasks.len() >= MAX_EMAIL_CONCURRENCY {
            process_batch(pool, account_id, &mut tasks).await?;
        }
    }

    // ✅ Process remaining tasks
    while !tasks.is_empty() {
        process_batch(pool, account_id, &mut tasks).await?;
    }

    // ✅ Update last_sync AFTER successful sync
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "UPDATE email_accounts SET last_sync = $1 WHERE id = $2"
    )
    .bind(now)
    .bind(account_id)
    .execute(pool)
    .await?;

    Ok(())
}

//////////////////////////////////////////////////
// BATCH INSERT
//////////////////////////////////////////////////

pub async fn process_batch<F>(
    pool: &PgPool,
    account_id: i32,
    tasks: &mut FuturesUnordered<F>,
) -> anyhow::Result<()>
where
    F: std::future::Future<
        Output = anyhow::Result<(String, String, String, String, String)>
    >,
{
    // ✅ Collect batch
    let mut batch: Vec<(String, String, String, String, String)> = vec![];

    for _ in 0..BATCH_SIZE {
        if let Some(res) = tasks.next().await {
            if let Ok(v) = res {
                batch.push(v);
            }
        } else {
            break;
        }
    }

    if batch.is_empty() {
        return Ok(());
    }

    // ✅ Build dynamic query
    let mut query = String::from(
        "INSERT INTO emails(gmail_id, sender, receiver, subject, body_encrypted, body_iv, account_id) VALUES "
    );

    for (i, _) in batch.iter().enumerate() {
        let idx = i * 7;

        query.push_str(&format!(
            "(${}, ${}, ${}, ${}, ${}, ${}, ${}),",
            idx + 1, idx + 2, idx + 3, idx + 4, idx + 5, idx + 6, idx + 7
        ));
    }

    query.pop(); // remove trailing comma
    query.push_str(" ON CONFLICT (account_id, gmail_id) DO NOTHING");

    // ✅ Bind values safely (correct types)
    let mut q = sqlx::query(&query);


    for (gmail_id, sender, receiver, subject, body) in batch.iter() {

        let (iv, encrypted_body) = encrypt(body)?;

        q = q
            .bind(gmail_id)
            .bind(sender)
            .bind(receiver)
            .bind(subject)
            .bind(encrypted_body) // 🔒 encrypted
            .bind(iv)             // 🔑 IV
            .bind(account_id);
    }

    // ✅ Execute ONCE
    q.execute(pool).await?;

    Ok(())
}