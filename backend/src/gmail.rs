use crate::prelude::*;
use crate::security::encryption::encrypt;


#[path = "models/email_request.rs"]
use crate::models::email_request::SendEmailRequest;
#[derive(Deserialize)]
pub struct OAuthQuery {
    code: String,
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
}

// LOGIN
pub async fn gmail_login() -> impl Responder {
    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"]
        .as_str()
        .unwrap();
    let redirect_uri = secrets["web"]["redirect_uris"][0]
        .as_str()
        .unwrap();
    let scope = "https://www.googleapis.com/auth/userinfo.email \
                 https://www.googleapis.com/auth/gmail.send \
                 https://www.googleapis.com/auth/gmail.modify \
                 https://www.googleapis.com/auth/gmail.readonly";
    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth\
        ?client_id={}\
        &redirect_uri={}\
        &response_type=code\
        &scope={}\
        &access_type=offline\
        &prompt=consent",
        client_id,
        redirect_uri,
        scope
    );

    HttpResponse::Found()
        .append_header(("Location", url))
        .finish()
}


//////////////////////////////////////////////////
// LOAD GOOGLE SECRETS
//////////////////////////////////////////////////

fn load_google_secrets() -> serde_json::Value {
    let data = fs::read_to_string("client_secret.json")
        .expect("Failed to read client_secret.json");

    serde_json::from_str(&data).unwrap()
}


// CALLBACK
pub async fn oauth_callback(
    pool: web::Data<PgPool>,
    query: web::Query<CallbackQuery>,
) -> impl Responder {

    let code = &query.code;

    let secrets = load_google_secrets();

    let client_id = secrets["web"]["client_id"]
    .as_str()
    .unwrap()
    .to_string();

    let client_secret = secrets["web"]["client_secret"]
        .as_str()
        .unwrap()
        .to_string();

    let redirect_uri = secrets["web"]["redirect_uris"][0]
        .as_str()
        .unwrap()
        .to_string();

    // 🔁 exchange code → tokens
    let res: Value = HTTP_CLIENT
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
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

    let access_token = res["access_token"].as_str().unwrap_or("");
    let refresh_token = res["refresh_token"].as_str().unwrap_or("");
    let expires_in = res["expires_in"].as_i64().unwrap_or(3600);
    let expiry = chrono::Utc::now() + chrono::Duration::seconds(expires_in);

    // 🔍 get user email
    let user_info: Value = HTTP_CLIENT
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let email = user_info["email"].as_str().unwrap_or("");

    let frontend = std::env::var("FRONTEND_URL")
    .unwrap_or("http://localhost:5173".to_string());

    let redirect = format!("{}/emails", frontend);

    HttpResponse::Found()
        .append_header(("Location", redirect))
        .finish();

    // 💾 SAVE TO DB (THIS WAS MISSING)
    match sqlx::query(
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
    .bind(email)
    .bind(access_token)
    .bind(refresh_token)
    .bind(expiry)
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => println!("✅ Account saved"),
        Err(e) => println!("❌ DB ERROR: {}", e),
    }

    HttpResponse::Ok().body(format!("Logged in: {}", email))
}


// 🔥 Global HTTP client
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .pool_max_idle_per_host(20)
        .build()
        .unwrap()
});



//////////////////////////////////////////////////
// SYNC ALL ACCOUNTS (PARALLEL)
//////////////////////////////////////////////////

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
                Err(e) => {
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

//////////////////////////////////////////////////
// FETCH IDS (PAGINATION + INCREMENTAL)
//////////////////////////////////////////////////

async fn fetch_ids(
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

//////////////////////////////////////////////////
// SYNC ACCOUNT
//////////////////////////////////////////////////

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

    // ✅ update last_sync AFTER success
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

async fn process_batch<F>(
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

//////////////////////////////////////////////////
// FETCH EMAIL DETAIL (FINAL FIXED)
//////////////////////////////////////////////////

async fn fetch_email_detail(
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
    let headers = &res["payload"]["headers"];

    let mut subject = String::new();
    let mut sender = String::new();
    let mut receiver = String::new();

    if let Some(arr) = headers.as_array() {
        for h in arr {
            let name = h.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let value = h.get("value").and_then(|v| v.as_str()).unwrap_or("");

            match name {
                "Subject" => subject = value.to_string(),
                "From" => sender = value.to_string(),
                "To" => receiver = value.to_string(),
                _ => {}
            }
        }
    }

    let snippet = res["snippet"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let body = extract_body(payload).unwrap_or_else(|| {
        res["snippet"].as_str().unwrap_or("").to_string()
    });
    
    Ok((
        msg_id.to_string(),
        sender,
        receiver,
        subject,
        body,
    ))
}


fn extract_body(payload: &Value) -> Option<String> {

    // ✅ 1. direct body
    if let Some(data) = payload["body"]["data"].as_str() {
        return Some(decode_base64(data));
    }

    // ✅ 2. parts (most important)
    if let Some(parts) = payload["parts"].as_array() {
        for part in parts {

            let mime = part["mimeType"].as_str().unwrap_or("");

            // 🔥 prefer HTML
            if mime == "text/html" {
                if let Some(data) = part["body"]["data"].as_str() {
                    return Some(decode_base64(data));
                }
            }

            // fallback text
            if mime == "text/plain" {
                if let Some(data) = part["body"]["data"].as_str() {
                    return Some(decode_base64(data));
                }
            }

            // 🔁 recursive (VERY IMPORTANT)
            if let Some(nested) = extract_body(part) {
                return Some(nested);
            }
        }
    }

    None
}


fn decode_base64(data: &str) -> String {
    let fixed = data.replace("-", "+").replace("_", "/");

    let decoded = base64::decode(fixed).unwrap_or_default();

    String::from_utf8_lossy(&decoded).to_string()
}

//////////////////////////////////////////////////
// REFRESH TOKEN
//////////////////////////////////////////////////

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

    if res.get("error").is_some() {
        return Err(anyhow::anyhow!("Token refresh failed"));
    }

    Ok(res["access_token"]
        .as_str()
        .unwrap_or("")
        .to_string())
}



#[post("/api/send")]
async fn send(
    data: web::Json<SendEmailRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {

    if data.to.trim().is_empty() || data.subject.trim().is_empty() {
        return HttpResponse::BadRequest().body("Recipient and Subject are required");
    }

    let account = sqlx::query("SELECT email, access_token FROM email_accounts WHERE id = $1")
        .bind(data.account_id)
        .fetch_one(pool.get_ref())
        .await;
    
    let (from_email, access_token) = match account {
        Ok(row) => {
            let email: String = row.get("email");
            let token: String = row.get("access_token");
            (email, token)
        },
        Err(_) => return HttpResponse::Unauthorized().body("Email account not found"),
    };
    let raw_email = format!(
        "From: {}\r\n\
To: {}\r\n\
Subject: {}\r\n\
MIME-Version: 1.0\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
{}",
        from_email.trim(),
        data.to.trim(),
        data.subject.trim(),
        data.body
    );

    let encoded = URL_SAFE_NO_PAD.encode(raw_email.as_bytes());

    let client = reqwest::Client::new();

    let res = client
        .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
        .bearer_auth(&access_token)
        .json(&serde_json::json!({ "raw": encoded }))
        .send()
        .await;

    match res {
        Ok(resp) => {
            let status = resp.status();
            let response_text = resp.text().await.unwrap_or_default();

            if status.is_success() {
                HttpResponse::Ok().body("Email sent ✅")
            } else {
                HttpResponse::InternalServerError()
                    .body(format!("Gmail rejected request: {}", response_text))
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().body("Failed to reach Gmail")
        }
    }
}


async fn get_access_token(pool: &PgPool) -> Option<String> {
    let row = sqlx::query("SELECT id, access_token, token_expiry, refresh_token FROM email_accounts WHERE is_active = true LIMIT 1"
    )
    .fetch_one(pool)
    .await
    .ok()?;

    let access_token: String = row.get("access_token");
    let expiry: Option<chrono::NaiveDateTime> = row.try_get("token_expiry").ok();
    let refresh_token: String = row.get("refresh_token");
    let account_id: i32 = row.get("id");

    // 🔥 CHECK EXPIRY
    if let Some(exp) = expiry {
        if exp < chrono::Utc::now().naive_utc() {

            let secrets = load_google_secrets();
            let client_id = secrets["web"]["client_id"].as_str().unwrap();
            let client_secret = secrets["web"]["client_secret"].as_str().unwrap();

            match refresh_access_token(client_id, client_secret, &refresh_token).await {
                Ok(new_token) => {
                    // save new token
                    let _ = sqlx::query(
                        "UPDATE email_accounts SET access_token=$1 WHERE id=$2"
                    )
                    .bind(&new_token)
                    .bind(account_id)
                    .execute(pool)
                    .await;

                    return Some(new_token);
                }
                Err(e) => {
                    return None;
                }
            }
        }
    }

    Some(access_token)
}


#[get("/accounts")]
async fn get_accounts(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query(
        "SELECT id, email FROM email_accounts"
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let accounts: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.get::<i32, _>("id"),
                        "email": r.get::<String, _>("email"),
                    })
                })
                .collect();

            HttpResponse::Ok().json(accounts) // ✅ IMPORTANT
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({ "error": "DB failure" })
            )
        }
    }
}