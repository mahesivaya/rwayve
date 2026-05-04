use crate::prelude::*;

use crate::email::oauth::{HTTP_CLIENT, load_google_secrets, refresh_access_token};
use crate::email::utils::extract_body;
use crate::security::encryption::encrypt;

use serde_json::Value;
use tokio::time::{Duration, sleep};

const BODY_CONCURRENCY: usize = 40;
const BODY_BATCH_SIZE: i64 = 200;
const ACCOUNTS_PER_ITERATION: i64 = 10;
const IDLE_SLEEP_SECS: u64 = 5;
const ERROR_SLEEP_SECS: u64 = 10;

pub fn start_body_worker(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            match run_iteration(&pool).await {
                Ok(0) => {
                    sleep(Duration::from_secs(IDLE_SLEEP_SECS)).await;
                }
                Ok(_) => {
                    // Work happened — loop immediately for next batch
                }
                Err(e) => {
                    println!("body_worker error: {:?}", e);
                    sleep(Duration::from_secs(ERROR_SLEEP_SECS)).await;
                }
            }
        }
    });
}

async fn run_iteration(pool: &PgPool) -> Result<usize> {
    let accounts = sqlx::query(
        r#"
        SELECT DISTINCT a.id, a.refresh_token
        FROM email_accounts a
        JOIN emails e ON e.account_id = a.id
        WHERE e.body_encrypted = ''
          AND a.refresh_token IS NOT NULL
          AND a.refresh_token <> ''
        LIMIT $1
        "#,
    )
    .bind(ACCOUNTS_PER_ITERATION)
    .fetch_all(pool)
    .await?;

    if accounts.is_empty() {
        return Ok(0);
    }

    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let client_secret = secrets["web"]["client_secret"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let mut handles = vec![];
    for row in accounts {
        let pool = pool.clone();
        let client_id = client_id.clone();
        let client_secret = client_secret.clone();
        let account_id: i32 = row.get("id");
        let refresh_token: String = row.get("refresh_token");

        handles.push(tokio::spawn(async move {
            process_account(
                &pool,
                account_id,
                &client_id,
                &client_secret,
                &refresh_token,
            )
            .await
        }));
    }

    let mut total = 0;
    for h in handles {
        match h.await {
            Ok(Ok(n)) => total += n,
            Ok(Err(e)) => println!("body_worker account error: {:?}", e),
            Err(e) => println!("body_worker join error: {:?}", e),
        }
    }

    Ok(total)
}

async fn process_account(
    pool: &PgPool,
    account_id: i32,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<usize> {
    let token = refresh_access_token(client_id, client_secret, refresh_token).await?;

    sqlx::query("UPDATE email_accounts SET access_token = $1 WHERE id = $2")
        .bind(&token)
        .bind(account_id)
        .execute(pool)
        .await?;

    let rows = sqlx::query(
        r#"
        SELECT id, gmail_id
        FROM emails
        WHERE account_id = $1 AND body_encrypted = ''
        ORDER BY id DESC
        LIMIT $2
        "#,
    )
    .bind(account_id)
    .bind(BODY_BATCH_SIZE)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(0);
    }

    let mut tasks: FuturesUnordered<_> = FuturesUnordered::new();
    let mut iter = rows.into_iter();
    let mut count = 0;

    fn spawn_fetch(
        tasks: &mut FuturesUnordered<
            std::pin::Pin<Box<dyn std::future::Future<Output = Result<(i32, String)>> + Send>>,
        >,
        token: String,
        id: i32,
        gmail_id: String,
    ) {
        tasks.push(Box::pin(
            async move { fetch_one(&token, id, &gmail_id).await },
        ));
    }

    // Prime the pump with up to BODY_CONCURRENCY in-flight requests.
    for row in iter.by_ref().take(BODY_CONCURRENCY) {
        let id: i32 = row.get("id");
        let gmail_id: String = row.get("gmail_id");
        spawn_fetch(&mut tasks, token.clone(), id, gmail_id);
    }

    while let Some(res) = tasks.next().await {
        match res {
            Ok((id, body)) => {
                if let Err(e) = update_body(pool, id, &body).await {
                    println!("body_worker update {} failed: {:?}", id, e);
                } else {
                    count += 1;
                }
            }
            Err(e) => println!("body_worker fetch failed: {:?}", e),
        }

        if let Some(row) = iter.next() {
            let id: i32 = row.get("id");
            let gmail_id: String = row.get("gmail_id");
            spawn_fetch(&mut tasks, token.clone(), id, gmail_id);
        }
    }

    Ok(count)
}

async fn fetch_one(token: &str, id: i32, gmail_id: &str) -> Result<(i32, String)> {
    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=full",
        gmail_id
    );

    let res: Value = HTTP_CLIENT
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?
        .json()
        .await?;

    let body = extract_body(&res["payload"])
        .unwrap_or_else(|| res["snippet"].as_str().unwrap_or("").to_string());

    Ok((id, body))
}

async fn update_body(pool: &PgPool, id: i32, body: &str) -> Result<()> {
    let (iv, encrypted) = encrypt(body)?;

    sqlx::query("UPDATE emails SET body_encrypted = $1, body_iv = $2 WHERE id = $3")
        .bind(encrypted)
        .bind(iv)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
