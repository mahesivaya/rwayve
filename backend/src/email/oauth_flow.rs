use crate::prelude::*;

use crate::email::oauth::{HTTP_CLIENT, load_google_secrets};
use crate::email::sync::sync_account_recent;
use crate::security::jwt::create_jwt_for_account;
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
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
                user_id = row.get("id");
                info!(target: "auth", user_id, "Google sign-in for existing user");
            }
            Ok(None) => {
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

    let frontend = match std::env::var("FRONTEND_URL") {
        Ok(v) => v,
        Err(e) => {
            error!("FRONTEND_URL missing: {:?}", e);
            return HttpResponse::InternalServerError().body("Server configuration error");
        }
    };

    let account_type: String = sqlx::query_scalar("SELECT account_type FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or_else(|_| "personal".to_string());

    let redirect = if is_signup {
        let session_token =
            create_jwt_for_account(user_id, email.to_string(), account_type.clone());
        let landing_path = if matches!(
            account_type.as_str(),
            "business" | "business_admin" | "project_admin"
        ) {
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
