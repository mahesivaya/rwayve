use crate::prelude::*;

use crate::email::oauth::{HTTP_CLIENT, load_google_secrets, refresh_access_token};

use serde_json::Value;

// (msg_id, sender, receiver, subject) — body is fetched later by body_worker
pub type EmailHeader = (String, String, String, String);

pub async fn fetch_headers_only(token: &str, msg_id: &str) -> Result<EmailHeader> {
    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=metadata&metadataHeaders=From&metadataHeaders=To&metadataHeaders=Subject",
        msg_id
    );

    let res: Value = HTTP_CLIENT
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .json()
        .await?;

    let (sender, receiver, subject) = extract_headers(&res);
    Ok((msg_id.to_string(), sender, receiver, subject))
}

pub async fn sync_all(pool: &PgPool) -> Result<()> {
    let rows = sqlx::query(
        "SELECT id, access_token, refresh_token, last_sync FROM email_accounts WHERE access_token IS NOT NULL"
    )
    .fetch_all(pool)
    .await?;

    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"]
        .as_str()
        .expect("client_id missing in google secrets");
    let client_secret = secrets["web"]["client_secret"]
        .as_str()
        .expect("client_secret missing in google secrets");

    let mut handles = vec![];

    for r in rows {
        let pool = pool.clone();

        let account_id: i32 = r.get("id");
        let refresh_token: String = r.get("refresh_token");
        let last_sync: Option<i64> = r.try_get("last_sync").ok();

        let client_id = client_id.to_string();
        let client_secret = client_secret.to_string();

        let handle = tokio::spawn(async move {
            let token = match refresh_access_token(&client_id, &client_secret, &refresh_token).await
            {
                Ok(t) => t,
                Err(_e) => {
                    return;
                }
            };

            let _ = sqlx::query("UPDATE email_accounts SET access_token=$1 WHERE id=$2")
                .bind(&token)
                .bind(account_id)
                .execute(&pool)
                .await;

            if let Err(e) = sync_account(&pool, account_id, &token, last_sync).await {
                println!("Sync error {}: {}", account_id, e);
            }
        });

        handles.push(handle);
    }

    for h in handles {
        let _ = h.await;
    }

    Ok(())
}

pub async fn fetch_ids(token: &str, last_sync: Option<i64>) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    let mut page_token: Option<String> = None;

    let query = if let Some(ts) = last_sync {
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

        page_token = res["nextPageToken"].as_str().map(|s| s.to_string());

        if page_token.is_none() {
            break;
        }
    }
    Ok(ids)
}

pub fn extract_headers(res: &Value) -> (String, String, String) {
    let mut sender: Option<String> = None;
    let mut receiver: Option<String> = None;
    let mut subject: Option<String> = None;

    fn walk_parts(
        node: &Value,
        sender: &mut Option<String>,
        receiver: &mut Option<String>,
        subject: &mut Option<String>,
    ) {
        if let Some(headers) = node["headers"].as_array() {
            for h in headers {
                let name = h["name"].as_str().unwrap_or("");
                let value = h["value"].as_str().unwrap_or("").to_string();

                match name {
                    "From" if sender.is_none() && !value.is_empty() => {
                        *sender = Some(value);
                    }
                    "To" if receiver.is_none() && !value.is_empty() => {
                        *receiver = Some(value);
                    }
                    "Subject" if subject.is_none() && !value.is_empty() => {
                        *subject = Some(value);
                    }
                    _ => {}
                }
            }
        }

        if let Some(parts) = node["parts"].as_array() {
            for part in parts {
                walk_parts(part, sender, receiver, subject);
            }
        }
    }

    walk_parts(&res["payload"], &mut sender, &mut receiver, &mut subject);

    let sender = sender.unwrap_or_else(|| "Unknown".to_string());
    let receiver = receiver.unwrap_or_else(|| "Unknown".to_string());
    let subject = subject.unwrap_or_else(|| "(No Subject)".to_string());

    (sender, receiver, subject)
}

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
        tasks.push(async move { fetch_headers_only(&token, &id).await });

        if tasks.len() >= MAX_EMAIL_CONCURRENCY {
            process_batch(pool, account_id, &mut tasks).await?;
        }
    }

    while !tasks.is_empty() {
        process_batch(pool, account_id, &mut tasks).await?;
    }

    let now = chrono::Utc::now().timestamp();

    sqlx::query("UPDATE email_accounts SET last_sync = $1 WHERE id = $2")
        .bind(now)
        .bind(account_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn process_batch<F>(
    pool: &PgPool,
    account_id: i32,
    tasks: &mut FuturesUnordered<F>,
) -> anyhow::Result<()>
where
    F: std::future::Future<Output = anyhow::Result<EmailHeader>>,
{
    let mut batch: Vec<EmailHeader> = vec![];

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

    // Insert headers with empty body sentinel — body_worker will fill these in.
    let mut query = String::from(
        "INSERT INTO emails(gmail_id, sender, receiver, subject, body_encrypted, body_iv, account_id) VALUES ",
    );

    for (i, _) in batch.iter().enumerate() {
        let idx = i * 7;
        query.push_str(&format!(
            "(${}, ${}, ${}, ${}, ${}, ${}, ${}),",
            idx + 1,
            idx + 2,
            idx + 3,
            idx + 4,
            idx + 5,
            idx + 6,
            idx + 7
        ));
    }

    query.pop();
    query.push_str(" ON CONFLICT (account_id, gmail_id) DO NOTHING");

    let mut q = sqlx::query(&query);

    for (gmail_id, sender, receiver, subject) in batch.iter() {
        q = q
            .bind(gmail_id)
            .bind(sender)
            .bind(receiver)
            .bind(subject)
            .bind("") // body_encrypted — empty until body_worker fills it
            .bind("") // body_iv
            .bind(account_id);
    }

    q.execute(pool).await?;

    Ok(())
}
