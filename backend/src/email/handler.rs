use crate::prelude::*;

use crate::email::attachments::save_email_attachments;
use crate::email::oauth::HTTP_CLIENT;
use crate::email::oauth::{load_google_secrets, refresh_access_token, try_load_google_secrets};
use crate::email::sync::sync_account_recent;
use crate::email::utils::{extract_attachments, extract_body};
use crate::models::email_request::SendEmailRequest;
use crate::security::encryption::{decrypt, encrypt};
use crate::security::jwt::{create_jwt_for_account, get_user_id_from_request};
use actix_web::{HttpResponse, Responder, get, web};
use base64::Engine;
use sqlx::PgPool;
use sqlx::Row;
use tracing::{error, info, instrument, warn};

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginQuery {
    pub token: Option<String>,
    pub mode: Option<String>,
}

// Sentinel placed in OAuth `state` for the signup flow. The callback uses it
// to know it must create a user before linking the Gmail account.
const SIGNUP_STATE: &str = "signup";

fn google_redirect_uri(req: &HttpRequest, secrets: &Value) -> String {
    if let Ok(uri) = std::env::var("GOOGLE_OAUTH_REDIRECT_URI") {
        let uri = uri.trim();
        if !uri.is_empty() {
            return uri.to_string();
        }
    }

    if let Ok(base_url) = std::env::var("BACKEND_URL") {
        let base_url = base_url.trim().trim_end_matches('/');
        if !base_url.is_empty() {
            return format!("{base_url}/oauth/callback");
        }
    }

    let connection = req.connection_info();
    let host = connection.host();
    if host.starts_with("localhost") || host.starts_with("127.0.0.1") || host.starts_with("[::1]") {
        return format!("{}://{host}/oauth/callback", connection.scheme());
    }

    secrets["web"]["redirect_uris"][0]
        .as_str()
        .expect("redirect_uris missing in google secrets")
        .to_string()
}

#[instrument(target = "gmail", skip(req, query))]
pub async fn gmail_login(req: HttpRequest, query: web::Query<LoginQuery>) -> impl Responder {
    let secrets = load_google_secrets();
    let client_id = secrets["web"]["client_id"]
        .as_str()
        .expect("client_id missing in google secrets");
    let redirect_uri = google_redirect_uri(&req, &secrets);

    let is_signup = query.mode.as_deref() == Some("signup");

    let state = if is_signup {
        SIGNUP_STATE.to_string()
    } else {
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
            warn!(target: "gmail", "gmail_login rejected: missing token");
            return HttpResponse::Unauthorized().body("Missing token");
        }

        token
    };

    info!(target: "gmail", signup = is_signup, "gmail oauth flow start");

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
        client_id, redirect_uri, scope, state
    );

    HttpResponse::Found()
        .append_header(("Location", url))
        .finish()
}

#[instrument(target = "gmail", skip(pool, query))]
pub async fn oauth_callback(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<CallbackQuery>,
) -> impl Responder {
    let code = &query.code;

    let secrets = load_google_secrets();

    let state = match &query.state {
        Some(t) => t,
        None => {
            warn!(target: "gmail", "oauth_callback rejected: missing state");
            return HttpResponse::Unauthorized().body("Missing state");
        }
    };

    let is_signup = state == SIGNUP_STATE;

    // For "connect existing account" flow we resolve user_id up front from
    // the JWT in state. For signup we don't know it until after we exchange
    // the code and learn the user's Google email.
    let mut user_id: i32 = 0;
    if !is_signup {
        let decoded = match crate::security::jwt::decode_jwt(state) {
            Some(d) => d,
            None => {
                warn!(target: "gmail", "oauth_callback rejected: invalid jwt state");
                return HttpResponse::Unauthorized().body("Invalid token");
            }
        };
        user_id = decoded.sub;
    }
    info!(target: "gmail", user_id, signup = is_signup, "oauth_callback exchanging code");

    let client_id = secrets["web"]["client_id"]
        .as_str()
        .expect("client_id missing in google secrets")
        .to_string();
    let client_secret = secrets["web"]["client_secret"]
        .as_str()
        .expect("client_secret missing in google secrets")
        .to_string();
    let redirect_uri = google_redirect_uri(&req, &secrets);

    // 🔁 exchange code → tokens
    let res: Value = HTTP_CLIENT
        .post(crate::external::google_token_url())
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
        .get(crate::external::google_userinfo_url())
        .bearer_auth(access_token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let email = user_info["email"].as_str().unwrap_or("");

    if email.is_empty() {
        warn!(target: "gmail", "oauth_callback: Google did not return an email");
        return HttpResponse::BadRequest().body("Google account did not expose an email address");
    }

    // 🆕 Signup branch: resolve or create the user before linking Gmail.
    let frontend_for_errors = std::env::var("FRONTEND_URL").unwrap_or_default();
    if is_signup {
        let existing =
            sqlx::query("SELECT id, auth_provider, account_type FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(pool.get_ref())
                .await;

        match existing {
            Ok(Some(row)) => {
                let provider: String = row.get("auth_provider");
                if provider != "google" {
                    warn!(
                        target: "auth",
                        "Google signup blocked: {} already registered with {}",
                        email, provider
                    );
                    let redirect = format!("{}/login?error=email_exists", frontend_for_errors);
                    return HttpResponse::Found()
                        .append_header(("Location", redirect))
                        .finish();
                }
                // Already a Google user — just sign them back in.
                user_id = row.get("id");
                info!(target: "auth", user_id, "Google sign-in for existing user");
            }
            Ok(None) => {
                // Brand-new user via Google.
                let insert = sqlx::query(
                    "INSERT INTO users (email, password, auth_provider) \
                     VALUES ($1, NULL, 'google') RETURNING id",
                )
                .bind(email)
                .fetch_one(pool.get_ref())
                .await;

                match insert {
                    Ok(row) => {
                        user_id = row.get("id");
                        info!(target: "auth", user_id, email, "Google signup created user");
                    }
                    Err(e) => {
                        error!(target: "auth", error = %e, "Google signup user insert failed");
                        return HttpResponse::InternalServerError()
                            .body("Failed to create account");
                    }
                }
            }
            Err(e) => {
                error!(target: "auth", error = %e, "Google signup user lookup failed");
                return HttpResponse::InternalServerError().body("Database error");
            }
        }
    }

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
            let id: i32 = row.get("id");
            info!(
                "Gmail account connected: {} (user_id={}, account_id={})",
                email, user_id, id
            );
            id
        }
        Err(e) => {
            error!("Failed to save Gmail account {}: {:?}", email, e);
            return HttpResponse::InternalServerError().body("Failed to save account");
        }
    };

    // ⚡ Prime the first inbox page immediately. The normal sync worker still
    // does the complete mailbox walk later, but this makes a newly connected
    // account visible in the UI within a few seconds instead of waiting for a
    // full mailbox scan.
    let pool_clone = pool.clone();
    let token_clone = access_token.to_string();
    actix_web::rt::spawn(async move {
        match sync_account_recent(pool_clone.get_ref(), account_id, &token_clone, 51).await {
            Ok(_) => info!(target: "gmail", user_id, account_id, "recent email sync primed"),
            Err(e) => {
                warn!(target: "gmail", user_id, account_id, error = ?e, "recent email sync failed")
            }
        }
    });

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
            Ok(n) => {
                info!(target: "scheduler", user_id, account_id, count = n, "calendar import done")
            }
            Err(e) => {
                warn!(target: "scheduler", user_id, account_id, error = %e, "calendar import failed")
            }
        }
    });

    info!(target: "gmail", user_id, signup = is_signup, "redirecting to frontend");

    // 🔁 Redirect AFTER saving
    let frontend = match std::env::var("FRONTEND_URL") {
        Ok(v) => v,
        Err(e) => {
            error!("FRONTEND_URL missing: {:?}", e);
            return HttpResponse::InternalServerError().body("Server configuration error");
        }
    };

    // Signup: mint a fresh JWT for the new user and land on /home.
    // Connect-existing: reuse the inbound JWT (which is `state`) and land on /emails.
    let account_type: String = sqlx::query_scalar("SELECT account_type FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or_else(|_| "personal".to_string());

    let redirect = if is_signup {
        let session_token =
            create_jwt_for_account(user_id, email.to_string(), account_type.clone());
        let landing_path = if account_type == "business" {
            "business-home"
        } else {
            "home"
        };
        format!(
            "{}/{landing_path}?signup=true&token={}",
            frontend, session_token
        )
    } else {
        format!("{}/emails?connected=true&token={}", frontend, state)
    };

    HttpResponse::Found()
        .append_header(("Location", redirect))
        .finish()
}

#[post("/send")]
#[instrument(target = "gmail", skip(req, data, pool), fields(to = %data.to))]
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

    info!(target: "gmail", user_id, account_id = data.account_id, "send email request");

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
        .post(crate::external::gmail_send_url())
        .bearer_auth(&access_token)
        .json(&serde_json::json!({ "raw": encoded }))
        .send()
        .await;

    match res {
        Ok(resp) => {
            let status = resp.status();
            let response_text = resp.text().await.unwrap_or_default();

            if status.is_success() {
                info!("Email sent to {} (user_id={})", data.to, user_id);
                HttpResponse::Ok().body("Email sent ✅")
            } else {
                warn!(
                    "Gmail rejected send to {} (status={}, body={})",
                    data.to, status, response_text
                );
                HttpResponse::InternalServerError()
                    .body(format!("Gmail rejected request: {}", response_text))
            }
        }
        Err(e) => {
            error!("Failed to connect to Gmail API: {}", e);
            HttpResponse::InternalServerError().body("Failed to reach Gmail")
        }
    }
}

#[get("/me")]
#[instrument(target = "http", skip(req, pool))]
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
    let result = sqlx::query("SELECT id, email, account_type FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(row)) => {
            let id: i32 = row.get("id");
            let email: String = row.get("email");
            let account_type: String = row.get("account_type");

            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "email": email,
                "account_type": account_type
            }))
        }

        // 🔥 USER DELETED → FORCE LOGOUT
        Ok(None) => {
            HttpResponse::Unauthorized().json(serde_json::json!({ "error": "User not found" }))
        }

        Err(e) => {
            error!(target: "db", error = %e, "get_me lookup failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/emails/{id}")]
#[instrument(target = "http", skip(pool))]
pub async fn get_email_by_id(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let email_id = path.into_inner();

    let result = sqlx::query(
        r#"
        SELECT e.id, e.subject, e.sender, e.receiver, e.body_encrypted, e.body_iv,
               e.attachments_checked
        FROM emails e
        JOIN email_accounts a ON e.account_id = a.id
        WHERE e.id = $1 AND a.user_id = $2
        "#,
    )
    .bind(email_id)
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let body_iv: String = row.get("body_iv");
            let body_encrypted: String = row.get("body_encrypted");

            // Newly synced emails carry empty body until body_worker fetches
            // them — return a placeholder instead of attempting to decrypt
            // (which would have panicked on an empty nonce).
            let body = if body_encrypted.is_empty() || body_iv.is_empty() {
                String::new()
            } else {
                match crate::security::encryption::decrypt(&body_iv, &body_encrypted) {
                    Ok(text) => text,
                    Err(e) => {
                        warn!(
                            target: "gmail",
                            email_id,
                            error = %e,
                            "email body decrypt failed; returning empty body so client can refetch"
                        );
                        String::new()
                    }
                }
            };

            HttpResponse::Ok().json(serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "subject": row.get::<Option<String>, _>("subject").unwrap_or_default(),
                "sender": row.get::<Option<String>, _>("sender").unwrap_or_default(),
                "receiver": row.get::<Option<String>, _>("receiver").unwrap_or_default(),
                "body": body,
                "attachments_checked": row.get::<Option<bool>, _>("attachments_checked").unwrap_or(false)
            }))
        }

        Ok(None) => HttpResponse::NotFound().body("Email not found"),

        Err(e) => {
            error!(target: "db", email_id, error = ?e, "get_email_by_id failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/emails/{id}/body")]
#[instrument(target = "gmail", skip(req, path, pool))]
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
        SELECT e.id, e.gmail_id, e.body_encrypted, e.body_iv, e.attachments_checked,
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
            error!(target: "db", user_id, email_id, error = ?e, "get_email_body lookup failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let body_encrypted: String = row.get("body_encrypted");
    let body_iv: String = row.get("body_iv");
    let attachments_checked: Option<bool> = row.get("attachments_checked");

    // Cached path: body already fetched.
    if !body_encrypted.is_empty() && !body_iv.is_empty() {
        match decrypt(&body_iv, &body_encrypted) {
            Ok(body) => {
                if attachments_checked.unwrap_or(false) {
                    return HttpResponse::Ok().json(serde_json::json!({ "body": body }));
                }

                info!(
                    target: "gmail",
                    email_id,
                    "cached email body has no attachment metadata; refreshing Gmail payload"
                );
            }
            Err(e) => {
                warn!(
                    target: "gmail",
                    email_id,
                    error = %e,
                    "cached email body decrypt failed; refetching from Gmail"
                );
            }
        }
    }

    // On-demand path: fetch from Gmail, encrypt, persist.
    let gmail_id: Option<String> = row.get("gmail_id");
    let account_id: i32 = row.get("account_id");
    let refresh_token: Option<String> = row.get("refresh_token");

    let gmail_id = match gmail_id.filter(|value| !value.trim().is_empty()) {
        Some(value) => value,
        None => {
            error!(target: "gmail", email_id, "email body request missing gmail_id");
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "Email is missing its Gmail message id. Re-sync this account."
            }));
        }
    };

    let refresh_token = match refresh_token.filter(|value| !value.trim().is_empty()) {
        Some(value) => value,
        None => {
            error!(target: "gmail", account_id, "email account missing refresh_token");
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "This Gmail account needs to be reconnected before Wayve can load message bodies."
            }));
        }
    };

    let secrets = match try_load_google_secrets() {
        Ok(secrets) => secrets,
        Err(e) => {
            error!(target: "gmail", error = %e, "google secrets unavailable for body fetch");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Google OAuth client secret is not configured"
            }));
        }
    };

    let client_id = match secrets["web"]["client_id"].as_str() {
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => {
            error!(target: "gmail", "google client_id missing for body fetch");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Google OAuth client id is not configured"
            }));
        }
    };

    let client_secret = match secrets["web"]["client_secret"].as_str() {
        Some(value) if !value.trim().is_empty() => value.to_string(),
        _ => {
            error!(target: "gmail", "google client_secret missing for body fetch");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Google OAuth client secret is not configured"
            }));
        }
    };

    let token = match refresh_access_token(&client_id, &client_secret, &refresh_token).await {
        Ok(t) => t,
        Err(e) => {
            error!(target: "gmail", account_id, error = ?e, "refresh_access_token failed");
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
                error!(target: "gmail", email_id, error = %e, "gmail body json parse failed");
                return HttpResponse::BadGateway().finish();
            }
        },
        Err(e) => {
            error!(target: "gmail", email_id, error = %e, "gmail body request failed");
            return HttpResponse::BadGateway().finish();
        }
    };

    let body = extract_body(&res["payload"])
        .unwrap_or_else(|| res["snippet"].as_str().unwrap_or("").to_string());
    let attachments = extract_attachments(&res["payload"]);

    match encrypt(&body) {
        Ok((iv, encrypted)) => {
            if let Err(e) =
                sqlx::query(
                    "UPDATE emails SET body_encrypted = $1, body_iv = $2, attachments_checked = true WHERE id = $3",
                )
                    .bind(&encrypted)
                    .bind(&iv)
                    .bind(email_id)
                    .execute(pool.get_ref())
                    .await
            {
                error!(target: "db", email_id, error = ?e, "persisting email body failed");
            }
        }
        Err(e) => {
            error!(target: "gmail", email_id, error = %e, "email body encrypt failed");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to secure email body: {}", e)
            }));
        }
    }

    save_email_attachments(
        pool.get_ref(),
        email_id,
        account_id,
        &gmail_id,
        &attachments,
    )
    .await;

    HttpResponse::Ok().json(serde_json::json!({ "body": body }))
}

#[post("/save-public-key")]
#[instrument(target = "auth", skip(req, pool, body))]
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
        Ok(_) => {
            info!(target: "auth", user_id, "public key saved");
            HttpResponse::Ok().body("Saved")
        }
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "save_public_key failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}
