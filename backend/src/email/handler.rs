use crate::prelude::*;


use crate::models::email_request::SendEmailRequest;
use crate::email::oauth::load_google_secrets;
use crate::email::oauth::HTTP_CLIENT;

use base64::Engine;

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
}


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
        Err(_e) => println!("❌ DB ERROR: {}", _e),
    }

    println!("🚀 Redirecting to frontend...");
    let frontend = std::env::var("FRONTEND_URL")
        .unwrap_or("http://localhost".to_string());

    let redirect = format!("{}/emails", frontend);

    return HttpResponse::Found()
        .append_header(("Location", redirect))
        .finish();
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
        Err(_e) => {
            HttpResponse::InternalServerError().body("Failed to reach Gmail")
        }
    }
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
        Err(_e) => {
            HttpResponse::InternalServerError().json(
                serde_json::json!({ "error": "DB failure" })
            )
        }
    }
}


#[get("/api/me")]
async fn get_me(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> impl Responder {

    // 🔥 1. Extract Authorization header
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({ "error": "Missing token" })
            )
        }
    };

    // 🔥 2. Extract token
    let token = auth_header.replace("Bearer ", "");

    // 🔥 3. Decode JWT
    let decoded = match crate::models::auth::decode_jwt(&token) {
        Some(d) => d,
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({ "error": "Invalid token" })
            )
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
            HttpResponse::Unauthorized().json(
                serde_json::json!({ "error": "User not found" })
            )
        }

        Err(e) => {
            println!("❌ DB ERROR: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}