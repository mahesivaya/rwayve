//! Microsoft (Outlook) OAuth HTTP handlers.
//!
//! - sign-in:        `GET  /outlook/login?mode=signup`
//! - mailbox connect: `POST /api/outlook/connect-url` (authenticated)
//! - shared callback: `GET  /oauth/outlook/callback`
//!
//! Token exchange/refresh and Graph mailbox sync live in `email::outlook`.

use crate::prelude::*;

use crate::email::outlook::{
    OUTLOOK_MAIL_SCOPE, OutlookCredentials, OutlookTokens, exchange_code, outlook_credentials,
    sync_outlook_account,
};
use crate::security::jwt::{auth_cookie, create_jwt_for_account, get_user_id_from_request};
use crate::security::oauth::{OAuthState, consume_state, store_state};
use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sqlx::PgPool;
use tracing::{error, info, instrument, warn};

/// OAuth `state` flow tags — distinct from the Google flows (and each other)
/// so a state minted for one purpose can't be replayed for another.
const OUTLOOK_FLOW_SIGNUP: &str = "outlook_signup";
const OUTLOOK_FLOW_CONNECT: &str = "outlook_connect";

#[derive(Deserialize)]
pub struct OutlookLoginQuery {
    pub mode: Option<String>,
}

#[derive(Deserialize)]
pub struct OutlookCallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Serialize)]
pub struct OutlookConnectUrlResponse {
    pub url: String,
}

fn require_credentials() -> std::result::Result<OutlookCredentials, HttpResponse> {
    outlook_credentials().ok_or_else(|| {
        error!(
            target: "auth",
            "Outlook OAuth env vars missing (OUTLOOK_CLIENT_ID / OUTLOOK_CLIENT_SECRET / OUTLOOK_REDIRECT_URI)"
        );
        HttpResponse::InternalServerError().body("Outlook OAuth is not configured")
    })
}

fn random_oauth_state() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn authorize_url(creds: &OutlookCredentials, scope: &str, state: &str) -> String {
    let endpoint = format!(
        "{}/oauth2/v2.0/authorize",
        crate::external::microsoft_authority()
    );
    let mut url = reqwest::Url::parse(&endpoint)
        .unwrap_or_else(|err| panic!("valid Microsoft OAuth URL: {err}"));
    url.query_pairs_mut()
        .append_pair("client_id", &creds.client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &creds.redirect_uri)
        .append_pair("response_mode", "query")
        .append_pair("scope", scope)
        // Force the consent screen so newly-added scopes are actually granted
        // instead of reusing a cached, narrower grant.
        .append_pair("prompt", "consent")
        .append_pair("state", state);
    url.to_string()
}

/// `GET /outlook/login?mode=signup` — kicks off Microsoft sign-in.
#[instrument(target = "auth", skip(query, pool))]
pub async fn outlook_login(
    query: web::Query<OutlookLoginQuery>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let creds = match require_credentials() {
        Ok(c) => c,
        Err(response) => return response,
    };

    if query.mode.as_deref() != Some("signup") {
        return HttpResponse::BadRequest()
            .body("Use POST /api/outlook/connect-url to connect a mailbox");
    }

    let state = random_oauth_state();
    if let Err(e) = store_state(&state, None, OUTLOOK_FLOW_SIGNUP, pool.get_ref()).await {
        error!(target: "auth", error = %e, "outlook oauth state store failed");
        return HttpResponse::InternalServerError().body("Failed to start OAuth flow");
    }

    info!(target: "auth", "outlook oauth sign-in flow start");
    HttpResponse::Found()
        .append_header(("Location", authorize_url(&creds, OUTLOOK_MAIL_SCOPE, &state)))
        .finish()
}

/// `POST /api/outlook/connect-url` — returns the Microsoft consent URL for
/// connecting the signed-in user's Outlook mailbox.
#[post("/outlook/connect-url")]
#[instrument(target = "auth", skip(req, pool))]
pub async fn outlook_connect_url(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => {
            warn!(target: "auth", "outlook connect-url rejected: missing token");
            return HttpResponse::Unauthorized().body("Missing token");
        }
    };

    let creds = match require_credentials() {
        Ok(c) => c,
        Err(response) => return response,
    };

    let state = random_oauth_state();
    if let Err(e) =
        store_state(&state, Some(user_id), OUTLOOK_FLOW_CONNECT, pool.get_ref()).await
    {
        error!(target: "auth", error = %e, "outlook connect state store failed");
        return HttpResponse::InternalServerError().body("Failed to start OAuth flow");
    }

    info!(target: "auth", user_id, "outlook mailbox connect flow start");
    HttpResponse::Ok().json(OutlookConnectUrlResponse {
        url: authorize_url(&creds, OUTLOOK_MAIL_SCOPE, &state),
    })
}

/// `GET /oauth/outlook/callback` — shared callback for sign-in and mailbox
/// connect; the consumed `state.flow` decides which path runs.
#[instrument(target = "auth", skip(pool, query))]
pub async fn outlook_callback(
    pool: web::Data<PgPool>,
    query: web::Query<OutlookCallbackQuery>,
) -> impl Responder {
    let creds = match require_credentials() {
        Ok(c) => c,
        Err(response) => return response,
    };

    let state = match &query.state {
        Some(s) => s,
        None => {
            warn!(target: "auth", "outlook_callback rejected: missing state");
            return HttpResponse::Unauthorized().body("Missing state");
        }
    };

    let oauth_state = match consume_state(state, pool.get_ref()).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            warn!(target: "auth", "outlook_callback rejected: invalid state");
            return HttpResponse::Unauthorized().body("Invalid OAuth state");
        }
        Err(e) => {
            error!(target: "auth", error = %e, "outlook oauth state lookup failed");
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let is_connect = oauth_state.flow == OUTLOOK_FLOW_CONNECT;
    if !is_connect && oauth_state.flow != OUTLOOK_FLOW_SIGNUP {
        warn!(target: "auth", "outlook_callback rejected: unexpected flow");
        return HttpResponse::Unauthorized().body("Invalid OAuth state");
    }

    // Always exchange with the mail scope so a signed-in account can sync and
    // send immediately, not just on an explicit "connect mailbox".
    let tokens = match exchange_code(&creds, &query.code, OUTLOOK_MAIL_SCOPE).await {
        Ok(t) => t,
        Err(e) => {
            warn!(target: "auth", error = ?e, "outlook token exchange failed");
            return HttpResponse::BadGateway().body("Microsoft sign-in failed");
        }
    };

    let email = match graph_user_email(&tokens.access_token).await {
        Ok(email) => email,
        Err(response) => return response,
    };

    let frontend = std::env::var("FRONTEND_URL").unwrap_or_default();

    if is_connect {
        connect_mailbox(pool.get_ref(), &oauth_state, &email, &tokens, &frontend).await
    } else {
        sign_in(pool.get_ref(), &email, &tokens, &frontend).await
    }
}

/// Normalizes a Microsoft guest UPN (e.g. `user_gmail.com#ext#@tenant...`)
/// back into a clean address (`user@gmail.com`).
fn sanitize_microsoft_email(raw: &str) -> String {
    let email = raw.trim().to_lowercase();
    if !email.contains("#ext#") {
        return email;
    }
    if let Some(prefix) = email.split('#').next()
        && let Some(idx) = prefix.rfind('_')
    {
        let (local, domain_part) = prefix.split_at(idx);
        if domain_part.len() > 1 {
            let clean_domain = &domain_part[1..];
            if clean_domain.contains('.') {
                return format!("{local}@{clean_domain}");
            }
        }
    }
    email
}

/// Reads the signed-in user's primary email from Microsoft Graph `/me`.
async fn graph_user_email(access_token: &str) -> std::result::Result<String, HttpResponse> {
    let url = format!("{}/v1.0/me", crate::external::microsoft_graph_base());
    let me: Value = match crate::email::oauth::HTTP_CLIENT
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
    {
        Ok(resp) => match resp.json().await {
            Ok(value) => value,
            Err(e) => {
                error!(target: "auth", error = %e, "outlook graph /me parse failed");
                return Err(HttpResponse::BadGateway().body("Invalid response from Microsoft"));
            }
        },
        Err(e) => {
            error!(target: "auth", error = %e, "outlook graph /me request failed");
            return Err(HttpResponse::BadGateway().body("Failed to reach Microsoft"));
        }
    };

    let raw_email = me["mail"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            me["otherMails"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
        })
        .or_else(|| me["userPrincipalName"].as_str())
        .unwrap_or_default();

    let email = sanitize_microsoft_email(raw_email);
    if email.is_empty() {
        warn!(target: "auth", "outlook_callback: Microsoft did not return an email");
        return Err(HttpResponse::BadRequest()
            .body("Microsoft account did not expose an email address"));
    }
    Ok(email)
}

/// Sign-in path: find or create the wayve account, link the mailbox, mint a
/// session, redirect.
async fn sign_in(
    pool: &PgPool,
    email: &str,
    tokens: &OutlookTokens,
    frontend: &str,
) -> HttpResponse {
    let existing =
        sqlx::query("SELECT id, auth_provider, account_type FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await;

    let (user_id, account_type): (i32, String) = match existing {
        Ok(Some(row)) => {
            let provider: String = row.get("auth_provider");
            if provider != "microsoft" {
                warn!(
                    target: "auth",
                    "Outlook sign-in blocked: {email} already registered with {provider}"
                );
                return HttpResponse::Found()
                    .append_header(("Location", format!("{frontend}/login?error=email_exists")))
                    .finish();
            }
            (row.get("id"), row.get("account_type"))
        }
        Ok(None) => {
            match sqlx::query(
                "INSERT INTO users (username, email, password, auth_provider) \
                 VALUES ($1, $2, NULL, 'microsoft') RETURNING id, account_type",
            )
            .bind(email)
            .bind(email)
            .fetch_one(pool)
            .await
            {
                Ok(row) => (row.get("id"), row.get("account_type")),
                Err(e) => {
                    error!(target: "auth", error = %e, "Outlook signup user insert failed");
                    return HttpResponse::InternalServerError().body("Failed to create account");
                }
            }
        }
        Err(e) => {
            error!(target: "auth", error = %e, "Outlook sign-in user lookup failed");
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    info!(target: "auth", user_id, "Outlook sign-in success");

    // Link the mailbox so the inbox is available right after signing in.
    if let Ok(account_id) = upsert_email_account(pool, user_id, email, tokens).await {
        info!(target: "auth", user_id, account_id, "Outlook mailbox linked during sign-in");
    }

    let token = create_jwt_for_account(user_id, email.to_string(), account_type.clone());
    let landing = if matches!(
        account_type.as_str(),
        "business" | "business_admin" | "project_admin"
    ) {
        "business-home"
    } else {
        "home"
    };

    HttpResponse::Found()
        .cookie(auth_cookie(token))
        .append_header(("Location", format!("{frontend}/{landing}#signup=true")))
        .finish()
}

/// Connect path: link the mailbox tokens to the signed-in user.
async fn connect_mailbox(
    pool: &PgPool,
    oauth_state: &OAuthState,
    email: &str,
    tokens: &OutlookTokens,
    frontend: &str,
) -> HttpResponse {
    let Some(user_id) = oauth_state.user_id else {
        warn!(target: "auth", "outlook connect rejected: state had no user");
        return HttpResponse::Unauthorized().body("Invalid OAuth state");
    };

    match upsert_email_account(pool, user_id, email, tokens).await {
        Ok(account_id) => {
            info!(target: "auth", user_id, account_id, "Outlook mailbox connected");
            HttpResponse::Found()
                .append_header(("Location", format!("{frontend}/emails#connected=true")))
                .finish()
        }
        Err(e) => {
            error!(target: "auth", user_id, error = %e, "outlook account link failed");
            HttpResponse::InternalServerError().body("Failed to save account")
        }
    }
}

/// Upserts the `email_accounts` row for an Outlook mailbox and primes an
/// initial Graph sync. Microsoft rotates refresh tokens, so an empty new
/// refresh token keeps the previously-stored one.
async fn upsert_email_account(
    pool: &PgPool,
    user_id: i32,
    email: &str,
    tokens: &OutlookTokens,
) -> Result<i32> {
    let expiry = (chrono::Utc::now() + chrono::Duration::seconds(tokens.expires_in)).naive_utc();
    let refresh_token = tokens.refresh_token.as_deref().unwrap_or("");

    let row = sqlx::query(
        r#"
        INSERT INTO email_accounts
          (email, user_id, access_token, refresh_token, token_expiry, is_active, provider)
        VALUES ($1, $2, $3, $4, $5, true, 'microsoft')
        ON CONFLICT (user_id, email) DO UPDATE SET
          access_token = EXCLUDED.access_token,
          token_expiry = EXCLUDED.token_expiry,
          provider = 'microsoft',
          is_active = true,
          refresh_token = COALESCE(
            NULLIF(EXCLUDED.refresh_token, ''),
            email_accounts.refresh_token
          )
        RETURNING id
        "#,
    )
    .bind(email)
    .bind(user_id)
    .bind(&tokens.access_token)
    .bind(refresh_token)
    .bind(expiry)
    .fetch_one(pool)
    .await?;

    let account_id: i32 = row.get("id");

    // Prime an initial sync without blocking the redirect.
    let pool = pool.clone();
    let access_token = tokens.access_token.clone();
    actix_web::rt::spawn(async move {
        if let Err(e) = sync_outlook_account(&pool, account_id, &access_token, None).await {
            warn!(target: "worker", account_id, error = ?e, "initial outlook sync failed");
        }
    });

    Ok(account_id)
}
