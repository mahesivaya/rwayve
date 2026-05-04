use crate::prelude::*;

use crate::email::oauth::HTTP_CLIENT;
use crate::email::oauth::{load_google_secrets, refresh_access_token};
use crate::email::utils::extract_body;
use crate::models::email_request::SendEmailRequest;
use crate::security::encryption::{decrypt, encrypt};
use crate::security::jwt::get_user_id_from_request;
use actix_web::{HttpResponse, Responder, get, web};
use base64::Engine;
use sqlx::PgPool;
use sqlx::Row;

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginQuery {
    pub token: Option<String>,
}

pub async fn gmail_login(req: HttpRequest, query: web::Query<LoginQuery>) -> impl Responder {
    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"].as_str().unwrap();
    let redirect_uri = secrets["web"]["redirect_uris"][0].as_str().unwrap();

    let token = if let Some(t) = &query.token {
        t.clone()
    } else {
        req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .unwrap_or("")
            .to_string()
    };

    if token.is_empty() {
        return HttpResponse::Unauthorized().body("Missing token");
    }

    let scope = "https://www.googleapis.com/auth/userinfo.email \
                 https://www.googleapis.com/auth/gmail.send \
                 https://www.googleapis.com/auth/gmail.modify \
                 https://www.googleapis.com/auth/gmail.readonly \
                 https://www.googleapis.com/auth/calendar.readonly";
    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth\
        ?client_id={}\
        &redirect_uri={}\
        &response_type=code\
        &scope={}\
        &access_type=offline\
        &prompt=consent\
        &state={}",
        client_id, redirect_uri, scope, token
    );

    HttpResponse::Found()
        .append_header(("Location", url))
        .finish()
}

pub async fn oauth_callback(
    pool: web::Data<PgPool>,
    query: web::Query<CallbackQuery>,
) -> impl Responder {
    let code = &query.code;

    let secrets = load_google_secrets();

    // 🔥 Extract token from state
    let token = match &query.state {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().body("Missing state"),
    };

    // 🔥 Decode JWT
    let decoded = match crate::security::jwt::decode_jwt(&token) {
        Some(d) => d,
        None => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let user_id = decoded.sub;

    let client_id = secrets["web"]["client_id"].as_str().unwrap().to_string();
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

    // 💾 SAVE TO DB FIRST ✅
    let account_id: i32 = match sqlx::query(
        r#"
        INSERT INTO email_accounts
        (email, user_id, access_token, refresh_token, token_expiry, is_active)
        VALUES ($1,$2,$3,$4,$5,true)
        ON CONFLICT (user_id, email)
        DO UPDATE SET
            access_token = EXCLUDED.access_token,
            token_expiry = EXCLUDED.token_expiry,
            refresh_token = COALESCE(
                NULLIF(EXCLUDED.refresh_token, ''),
                email_accounts.refresh_token
            )
        RETURNING id
        "#,
    )
    .bind(email)
    .bind(user_id)
    .bind(access_token)
    .bind(refresh_token)
    .bind(expiry)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => {
            println!("✅ Account saved");
            row.get("id")
        }
        Err(e) => {
            println!("❌ DB ERROR: {}", e);
            return HttpResponse::InternalServerError().body("Failed to save account");
        }
    };

    // 📅 Import Google Calendar events in the background — best-effort, do not
    // block redirect if calendar scope was denied or API call fails.
    let pool_clone = pool.clone();
    let token_clone = access_token.to_string();
    actix_web::rt::spawn(async move {
        match crate::scheduler::google_calendar::import_upcoming_events(
            pool_clone.get_ref(),
            user_id,
            account_id,
            &token_clone,
        )
        .await
        {
            Ok(n) => println!("✅ Imported {} calendar events", n),
            Err(e) => println!("⚠️ Calendar import failed: {}", e),
        }
    });

    println!("🚀 Redirecting to frontend...");

    // 🔁 Redirect AFTER saving
    let frontend = std::env::var("FRONTEND_URL").unwrap_or("http://localhost:5173".to_string());

    let redirect = format!("{}/emails?connected=true&token={}", frontend, token);

    HttpResponse::Found()
        .append_header(("Location", redirect))
        .finish()
}

#[post("/send")]
async fn send(
    req: HttpRequest,
    data: web::Json<SendEmailRequest>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    if data.to.trim().is_empty() || data.subject.trim().is_empty() {
        return HttpResponse::BadRequest().body("Recipient and Subject are required");
    }

    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let account = sqlx::query(
        "SELECT email, access_token FROM email_accounts 
        WHERE id = $1 AND user_id = $2",
    )
    .bind(data.account_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await;
    let (from_email, access_token) = match account {
        Ok(row) => {
            let email: String = row.get("email");
            let token: String = row.get("access_token");
            (email, token)
        }
        Err(_) => return HttpResponse::Unauthorized().body("Email account not found"),
    };
    let raw_email = format!(
        "From: {}\r\n\
    To: {}\r\n\
    Subject: {}\r\n\
    MIME-Version: 1.0\r\n\
    Content-Type: text/plain; charset=\"UTF-8\"\r\n\
    Content-Transfer-Encoding: 7bit\r\n\
    \r\n\
    {}",
        from_email.trim(),
        data.to.trim(),
        data.subject.trim(),
        data.body.replace("\n", "\r\n")
    );

    let encoded = base64::engine::general_purpose::URL_SAFE.encode(raw_email.as_bytes());

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
        Err(_e) => HttpResponse::InternalServerError().body("Failed to reach Gmail"),
    }
}

#[get("/me")]
async fn get_me(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    // 🔥 1. Extract Authorization header
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "Missing token" }));
        }
    };

    // 🔥 2. Extract token
    let token = auth_header.replace("Bearer ", "");

    // 🔥 3. Decode JWT
    let decoded = match crate::security::jwt::decode_jwt(&token) {
        Some(d) => d,
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "Invalid token" }));
        }
    };

    let user_id = decoded.sub;

    // 🔥 4. Check DB (THIS is the key fix)
    let result = sqlx::query("SELECT id, email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(row)) => {
            let id: i32 = row.get("id");
            let email: String = row.get("email");

            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "email": email
            }))
        }

        // 🔥 USER DELETED → FORCE LOGOUT
        Ok(None) => {
            HttpResponse::Unauthorized().json(serde_json::json!({ "error": "User not found" }))
        }

        Err(e) => {
            println!("❌ DB ERROR: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/emails/{id}")]
pub async fn get_email_by_id(pool: web::Data<PgPool>, path: web::Path<i32>) -> impl Responder {
    let email_id = path.into_inner();

    let result = sqlx::query(
        r#"
        SELECT id, subject, sender, receiver, body_encrypted, body_iv
        FROM emails
        WHERE id = $1
        "#,
    )
    .bind(email_id) // ✅ CORRECT
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let body_iv: String = row.get("body_iv");
            let body_encrypted: String = row.get("body_encrypted");

            let body = match crate::security::encryption::decrypt(&body_iv, &body_encrypted) {
                Ok(text) => text,
                Err(_) => "Failed to decrypt".to_string(),
            };

            HttpResponse::Ok().json(serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "subject": row.get::<String, _>("subject"),
                "sender": row.get::<String, _>("sender"),
                "receiver": row.get::<String, _>("receiver"),
                "body": body
            }))
        }

        Ok(None) => HttpResponse::NotFound().body("Email not found"),

        Err(e) => {
            println!("❌ DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/emails/{id}/body")]
pub async fn get_email_body(
    req: HttpRequest,
    path: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let email_id = path.into_inner();

    let row = sqlx::query(
        r#"
        SELECT e.id, e.gmail_id, e.body_encrypted, e.body_iv,
               a.id AS account_id, a.refresh_token
        FROM emails e
        JOIN email_accounts a ON e.account_id = a.id
        WHERE e.id = $1 AND a.user_id = $2
        "#,
    )
    .bind(email_id)
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(e) => {
            println!("get_email_body DB error: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let body_encrypted: String = row.get("body_encrypted");
    let body_iv: String = row.get("body_iv");

    // Cached path: body already fetched.
    if !body_encrypted.is_empty() {
        return match decrypt(&body_iv, &body_encrypted) {
            Ok(body) => HttpResponse::Ok().json(serde_json::json!({ "body": body })),
            Err(e) => {
                println!("get_email_body decrypt error: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        };
    }

    // On-demand path: fetch from Gmail, encrypt, persist.
    let gmail_id: String = row.get("gmail_id");
    let account_id: i32 = row.get("account_id");
    let refresh_token: String = row.get("refresh_token");

    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let client_secret = secrets["web"]["client_secret"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let token = match refresh_access_token(&client_id, &client_secret, &refresh_token).await {
        Ok(t) => t,
        Err(e) => {
            println!("get_email_body token refresh error: {:?}", e);
            return HttpResponse::BadGateway().finish();
        }
    };

    let _ = sqlx::query("UPDATE email_accounts SET access_token = $1 WHERE id = $2")
        .bind(&token)
        .bind(account_id)
        .execute(pool.get_ref())
        .await;

    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=full",
        gmail_id
    );

    let res: Value = match HTTP_CLIENT.get(&url).bearer_auth(&token).send().await {
        Ok(r) => match r.json().await {
            Ok(v) => v,
            Err(e) => {
                println!("get_email_body Gmail JSON error: {:?}", e);
                return HttpResponse::BadGateway().finish();
            }
        },
        Err(e) => {
            println!("get_email_body Gmail request error: {:?}", e);
            return HttpResponse::BadGateway().finish();
        }
    };

    let body = extract_body(&res["payload"])
        .unwrap_or_else(|| res["snippet"].as_str().unwrap_or("").to_string());

    if let Ok((iv, encrypted)) = encrypt(&body) {
        let _ = sqlx::query("UPDATE emails SET body_encrypted = $1, body_iv = $2 WHERE id = $3")
            .bind(&encrypted)
            .bind(&iv)
            .bind(email_id)
            .execute(pool.get_ref())
            .await;
    }

    HttpResponse::Ok().json(serde_json::json!({ "body": body }))
}

#[post("/save-public-key")]
async fn save_public_key(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().body("Invalid token"),
    };

    let public_key = body["public_key"].to_string();

    let res = sqlx::query("UPDATE users SET public_key = $1 WHERE id = $2")
        .bind(public_key)
        .bind(user_id)
        .execute(pool.get_ref())
        .await;

    match res {
        Ok(_) => HttpResponse::Ok().body("Saved"),
        Err(e) => {
            println!("DB error: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
