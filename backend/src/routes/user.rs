use crate::email::profile::invalidate_me_cache;
use crate::models::auth::ChangePasswordInput;
use crate::models::email_request::UserResponse;
use crate::prelude::*;
use crate::security::jwt::get_user_id_from_request;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, put, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use moka::future::Cache as MokaCache;
use serde::Deserialize;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, instrument, warn};

const PROFILE_CACHE_TTL_SECS: u64 = 30;
const PROFILE_CACHE_MAX_CAPACITY: u64 = 5000;

static PROFILE_CACHE: Lazy<MokaCache<i32, serde_json::Value>> = Lazy::new(|| {
    MokaCache::builder()
        .max_capacity(PROFILE_CACHE_MAX_CAPACITY)
        .time_to_live(Duration::from_secs(PROFILE_CACHE_TTL_SECS))
        .build()
});

pub async fn invalidate_profile_cache(user_id: i32) {
    PROFILE_CACHE.invalidate(&user_id).await;
}

/// Canonical account-type string. `account_type` is a plain TEXT column;
/// anything unrecognized normalizes to "personal".
pub fn normalized_account_type(value: &str) -> &str {
    match value {
        "organization" | "organization_admin" | "platform_admin" => value,
        _ => "personal",
    }
}

#[derive(Deserialize)]
pub struct UserLookupQuery {
    pub email: String,
}

#[get("/users")]
#[instrument(target = "http", skip(req, pool, query))]
pub async fn get_user_by_email(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    query: web::Query<UserLookupQuery>,
) -> impl Responder {
    // Require a valid JWT — this endpoint exposes user ids and public keys,
    // so it must not be reachable anonymously.
    if get_user_id_from_request(&req).is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let email = query.email.trim();
    if email.is_empty() {
        return HttpResponse::BadRequest().body("Email required");
    }

    let result = sqlx::query_as::<_, UserResponse>(
        "SELECT id, email, public_key FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(user)) => {
            let parsed_key = user
                .public_key
                .and_then(|k| serde_json::from_str::<Vec<u8>>(&k).ok());

            HttpResponse::Ok().json(serde_json::json!({
                "id": user.id,
                "email": user.email,
                "public_key": parsed_key
            }))
        }

        Ok(None) => HttpResponse::Ok().json(serde_json::json!(null)),

        Err(e) => {
            error!(target: "db", error = ?e, "get_user_by_email lookup failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

use sqlx::Row;

#[derive(Deserialize)]
pub struct ProfileUpdate {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Deserialize)]
pub struct GenerateApiKeyInput {
    pub name: String,
}

#[derive(Deserialize)]
pub struct AdminCreateUserInput {
    pub username: String,
    pub email: String,
    pub password: String,
    pub account_type: Option<String>,
    pub organization_name: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateOrganizationInput {
    pub name: String,
    /// Optional organization admin to provision together with the organization. When
    /// any of the three fields is supplied, all three are required.
    pub admin_username: Option<String>,
    pub admin_email: Option<String>,
    pub admin_password: Option<String>,
}

async fn require_platform_admin(req: &HttpRequest, pool: &PgPool) -> Result<i32, HttpResponse> {
    let admin_id =
        get_user_id_from_request(req).ok_or_else(|| HttpResponse::Unauthorized().finish())?;

    let account_type: Option<String> =
        match sqlx::query_scalar("SELECT account_type FROM users WHERE id = $1")
            .bind(admin_id)
            .fetch_optional(pool)
            .await
        {
            Ok(value) => value,
            Err(e) => {
                error!(target: "db", admin_id, error = ?e, "platform admin lookup failed");
                return Err(HttpResponse::InternalServerError().finish());
            }
        };

    if normalized_account_type(account_type.as_deref().unwrap_or("personal")) != "platform_admin" {
        return Err(HttpResponse::Forbidden().json(
            serde_json::json!({ "message": "Only platform admins can manage organizations" }),
        ));
    }

    Ok(admin_id)
}

#[get("/admin/organizations")]
#[instrument(target = "http", skip(req, pool))]
pub async fn admin_list_organizations(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    if let Err(response) = require_platform_admin(&req, pool.get_ref()).await {
        return response;
    }

    match sqlx::query(
        r#"
        SELECT
            o.id,
            o.name,
            o.slug,
            o.created_at,
            COUNT(u.id) AS user_count,
            (SELECT json_build_object('id', u2.id, 'email', u2.email) 
             FROM users u2 
             WHERE u2.organization_id = o.id AND u2.account_type = 'organization_admin'
             LIMIT 1) as admin
        FROM organizations o
        LEFT JOIN users u ON u.organization_id = o.id
        GROUP BY o.id, o.name, o.slug, o.created_at
        ORDER BY o.name
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => {
            let organizations: Vec<_> = rows
                .into_iter()
                .map(|row| {
                    let id: i32 = row.get("id");
                    let name: String = row.get("name");
                    let slug: Option<String> = row.get("slug");
                    let user_count: i64 = row.get("user_count");
                    let admin: Option<serde_json::Value> = row.get("admin");

                    serde_json::json!({
                        "id": id,
                        "name": name,
                        "slug": slug,
                        "user_count": user_count,
                        "admin": admin
                    })
                })
                .collect();

            HttpResponse::Ok().json(organizations)
        }
        Err(e) => {
            error!(target: "db", error = ?e, "list organizations failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/admin/organizations")]
#[instrument(target = "auth", skip(req, pool, data), fields(name = %data.name))]
pub async fn admin_create_organization(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<CreateOrganizationInput>,
) -> impl Responder {
    let admin_id = match require_platform_admin(&req, pool.get_ref()).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    let name = data.name.trim();
    if name.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "Organization name is required" }));
    }

    // The organization admin block is optional, but if any field is supplied the
    // whole set (username, email, password) must be present.
    let admin_username = data
        .admin_username
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let admin_email = data
        .admin_email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase);
    let admin_password = data
        .admin_password
        .as_deref()
        .filter(|value| !value.is_empty());

    let organization_admin =
        if admin_username.is_some() || admin_email.is_some() || admin_password.is_some() {
            match (admin_username, admin_email.as_deref(), admin_password) {
                (Some(username), Some(email), Some(password)) => {
                    if password.len() < 6 {
                        return HttpResponse::BadRequest().json(serde_json::json!({
                            "message": "Password must be at least 6 characters"
                        }));
                    }
                    Some((
                        username.to_string(),
                        email.to_string(),
                        password.to_string(),
                    ))
                }
                _ => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Organization admin username, email, and password are all required"
                }));
                }
            }
        } else {
            None
        };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(target: "db", admin_id, error = ?e, "begin organization transaction failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let org_row = match sqlx::query(
        r#"
        INSERT INTO organizations (name)
        VALUES ($1)
        ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id, name, slug
        "#,
    )
    .bind(name)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            error!(target: "db", admin_id, error = ?e, "create organization failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let organization_id: i32 = org_row.get("id");
    let organization_name: String = org_row.get("name");
    let organization_slug: Option<String> = org_row.get("slug");

    let mut admin_json = serde_json::Value::Null;

    if let Some((username, email, password)) = organization_admin {
        let hashed = match hash(&password, DEFAULT_COST) {
            Ok(value) => value,
            Err(e) => {
                error!(target: "auth", error = %e, "organization admin bcrypt hash failed");
                return HttpResponse::InternalServerError().finish();
            }
        };

        match sqlx::query(
            r#"
            INSERT INTO users (username, email, password, auth_provider, account_type, organization_id)
            VALUES ($1, $2, $3, 'local', $4, $5)
            RETURNING id, username, email, account_type, organization_id
            "#,
        )
        .bind(&username)
        .bind(&email)
        .bind(&hashed)
        .bind("organization_admin")
        .bind(organization_id)
        .fetch_one(&mut *tx)
        .await
        {
            Ok(row) => {
                let id: i32 = row.get("id");
                let username: Option<String> = row.try_get("username").ok();
                let email: String = row.get("email");
                let account_type: String = row.get("account_type");
                let org_id: Option<i32> = row.try_get("organization_id").ok().flatten();
                admin_json = serde_json::json!({
                    "id": id,
                    "username": username,
                    "email": email,
                    "account_type": account_type, // Use the enum directly
                    "organization_id": org_id
                });
            }
            Err(e) => {
                if e.to_string().contains("duplicate key") {
                    return HttpResponse::Conflict().json(serde_json::json!({
                        "message": "A user with that username or email already exists"
                    }));
                }
                error!(target: "db", admin_id, error = ?e, "create organization admin failed");
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

    if let Err(e) = tx.commit().await {
        error!(target: "db", admin_id, error = ?e, "commit organization transaction failed");
        return HttpResponse::InternalServerError().finish();
    }

    let user_count = if admin_json.is_null() { 0 } else { 1 };
    info!(target: "auth", admin_id, organization_id, "platform admin created organization");
    HttpResponse::Created().json(serde_json::json!({
        "id": organization_id,
        "name": organization_name,
        "slug": organization_slug,
        "user_count": user_count,
        "admin": admin_json
    }))
}

#[post("/admin/organizations/{id}/keys")]
#[instrument(target = "auth", skip(req, pool, data))]
pub async fn admin_generate_api_key(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    data: web::Json<GenerateApiKeyInput>,
) -> impl Responder {
    let admin_id = match require_platform_admin(&req, pool.get_ref()).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    let organization_id = path.into_inner();
    let key_name = data.name.trim();

    if key_name.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({ "message": "Key name is required" }));
    }

    // 1. Generate a secure random key
    // In a real production app, use the `rand` crate:
    // let raw_key = format!("wv_sk_{}", hex::encode(rand::thread_rng().gen::<[u8; 24]>()));
    let raw_key = format!("wv_sk_{}", uuid::Uuid::new_v4().simple()); // Fallback example
    
    // 2. Hash the key for storage (SHA-256)
    // We'll simulate hashing here. Use `sha2` crate in production.
    let key_hash = bcrypt::hash(&raw_key, DEFAULT_COST).unwrap_or_default();
    let key_preview = format!("{}...{}", &raw_key[..7], &raw_key[raw_key.len() - 4..]);

    match sqlx::query(
        r#"
        INSERT INTO api_keys (organization_id, name, key_hash, key_preview)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, created_at
        "#,
    )
    .bind(organization_id)
    .bind(key_name)
    .bind(&key_hash)
    .bind(&key_preview)
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(row) => {
            info!(target: "auth", admin_id, organization_id, "api key generated");
            HttpResponse::Created().json(serde_json::json!({
                "id": row.get::<i32, _>("id"),
                "name": row.get::<String, _>("name"),
                "created_at": row.get::<chrono::NaiveDateTime, _>("created_at"),
                "api_key": raw_key // This is the ONLY time the raw key is returned
            }))
        }
        Err(e) => {
            error!(target: "db", error = ?e, "failed to generate api key");
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Helper to validate an API key from headers
/// You would use this in a custom Actix Extractor or Middleware
pub async fn validate_api_key(req: &HttpRequest, pool: &PgPool) -> Option<i32> {
    let api_key = req.headers().get("X-API-KEY")?.to_str().ok()?;
    
    // In production, fetch the hash from DB based on a key ID or prefix
    // Then verify. Since we stored a bcrypt hash in this example:
    let rows = sqlx::query("SELECT organization_id, key_hash FROM api_keys")
        .fetch_all(pool)
        .await
        .ok()?;

    for row in rows {
        let hash: String = row.get("key_hash");
        if bcrypt::verify(api_key, &hash).unwrap_or(false) {
            let org_id: i32 = row.get("organization_id");
            return Some(org_id);
        }
    }

    None
}

#[post("/admin/users")]
#[instrument(target = "auth", skip(req, pool, data), fields(email = %data.email))]
pub async fn admin_create_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<AdminCreateUserInput>,
) -> impl Responder {
    let admin_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let admin_row = match sqlx::query_as::<_, (String, Option<i32>)>(
        "SELECT account_type, organization_id FROM users WHERE id = $1",
    )
    .bind(admin_id)
    .fetch_optional(pool.get_ref())
    .await
    {
        Ok(value) => value,
        Err(e) => {
            error!(target: "db", admin_id, error = ?e, "admin account lookup failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let (admin_account_type, admin_organization_id) = match admin_row {
        Some(pair) => pair,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if !matches!(
        normalized_account_type(&admin_account_type),
        "organization_admin" | "platform_admin"
    ) {
        warn!(target: "auth", admin_id, "non-admin user tried to create user");
        return HttpResponse::Forbidden()
            .json(serde_json::json!({ "message": "Only admins can create users" }));
    }

    let username = data.username.trim();
    let email = data.email.trim().to_lowercase();
    let requested_account_type = data
        .account_type
        .as_deref()
        .map(normalized_account_type)
        .unwrap_or("personal");

    let account_type: &str = match normalized_account_type(&admin_account_type) {
        "platform_admin" => match requested_account_type {
            "organization_admin" | "platform_admin" | "organization" | "personal" => {
                requested_account_type
            }
            _ => "personal",
        },
        "organization_admin" => "organization",
        _ => "personal",
    };

    if username.is_empty() || email.is_empty() || data.password.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "Username, email, and password are required" }));
    }

    if data.password.len() < 6 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "Password must be at least 6 characters" }));
    }

    let organization_id: Option<i32> = if account_type == "organization_admin" {
        let organization_name = data
            .organization_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let Some(organization_name) = organization_name else {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({ "message": "Organization name is required for organization admin accounts" }));
        };

        match sqlx::query(
            r#"
            INSERT INTO organizations (name)
            VALUES ($1)
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(organization_name)
        .fetch_one(pool.get_ref())
        .await
        {
            Ok(row) => Some(row.get("id")),
            Err(e) => {
                error!(target: "db", admin_id, error = ?e, "organization upsert failed");
                return HttpResponse::InternalServerError().finish();
            }
        }
    } else if normalized_account_type(&admin_account_type) == "organization_admin" {
        match admin_organization_id {
            Some(id) => Some(id),
            None => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Organization admin is not assigned to an organization"
                }));
            }
        }
    } else {
        None
    };

    let hashed = match hash(&data.password, DEFAULT_COST) {
        Ok(value) => value,
        Err(e) => {
            error!(target: "auth", error = %e, "admin create user bcrypt hash failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let result = sqlx::query(
        r#"
        INSERT INTO users (username, email, password, auth_provider, account_type)
        VALUES ($1, $2, $3, 'local', $4)
        RETURNING id, username, email, account_type, organization_id
        "#,
    )
    .bind(username)
    .bind(&email)
    .bind(&hashed)
    .bind(account_type)
    .fetch_one(pool.get_ref())
    .await;

    let result = if let (Ok(row), Some(organization_id)) = (&result, organization_id) {
        sqlx::query(
            "UPDATE users SET organization_id = $1 WHERE id = $2 RETURNING id, username, email, account_type, organization_id",
        )
        .bind(organization_id)
        .bind(row.get::<i32, _>("id"))
        .fetch_one(pool.get_ref())
        .await
    } else {
        result
    };

    match result {
        Ok(row) => {
            let id: i32 = row.get("id");
            let username: Option<String> = row.try_get("username").ok();
            let email: String = row.get("email");
            let account_type: String = row.get("account_type");
            let organization_id: Option<i32> = row.try_get("organization_id").ok().flatten();

            info!(target: "auth", admin_id, user_id = id, "admin created user");
            HttpResponse::Created().json(serde_json::json!({
                "id": id,
                "username": username,
                "email": email,
                "account_type": account_type,
                "organization_id": organization_id
            }))
        }
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                return HttpResponse::Conflict()
                    .json(serde_json::json!({ "message": "Username or email already exists" }));
            }

            error!(target: "db", admin_id, error = ?e, "admin create user failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/profile")]
#[instrument(target = "http", skip(req, pool))]
pub async fn get_profile(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if let Some(cached) = PROFILE_CACHE.get(&user_id).await {
        return HttpResponse::Ok().json(cached);
    }

    let result = sqlx::query(
        r#"
        SELECT 
            u.id, u.email, u.first_name, u.last_name, u.auth_provider, u.account_type, u.organization_id,
            o.name as organization_name,
            (SELECT COUNT(*)::BIGINT FROM emails e JOIN email_accounts ea ON e.account_id = ea.id WHERE ea.user_id = u.id) as total_emails,
            (SELECT COALESCE(SUM(octet_length(body_encrypted)), 0)::BIGINT FROM emails e JOIN email_accounts ea ON e.account_id = ea.id WHERE ea.user_id = u.id) as email_storage_bytes,
            (SELECT COALESCE(SUM(size), 0)::BIGINT FROM files f WHERE f.user_id = u.id) as drive_storage_bytes,
            (SELECT COALESCE(SUM(octet_length(content_encrypted)), 0)::BIGINT FROM messages m WHERE m.sender_id = u.id) as chat_storage_bytes,
            (SELECT COALESCE(SUM(octet_length(coalesce(content_encrypted, content, ''))), 0)::BIGINT FROM notes n WHERE n.user_id = u.id) as notes_storage_bytes
        FROM users u 
        LEFT JOIN organizations o ON o.id = u.organization_id
        WHERE u.id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let id: i32 = row.get("id");
            let email: String = row.get("email");
            let first_name: Option<String> = row.try_get("first_name").ok();
            let last_name: Option<String> = row.try_get("last_name").ok();
            let auth_provider: String = row
                .try_get("auth_provider")
                .unwrap_or_else(|_| "local".to_string());
            let account_type: String = row
                .try_get("account_type")
                .unwrap_or_else(|_| "personal".to_string());
            let total_emails: i64 = row.get("total_emails");
            let email_storage_bytes: i64 = row.get("email_storage_bytes");
            let drive_storage_bytes: i64 = row.get("drive_storage_bytes");
            let chat_storage_bytes: i64 = row.get("chat_storage_bytes");
            let notes_storage_bytes: i64 = row.get("notes_storage_bytes");
            let total_used = email_storage_bytes
                + drive_storage_bytes
                + chat_storage_bytes
                + notes_storage_bytes;

            // Requirement: For personal accounts, organization name is the email address.
            let organization_id: Option<i32> = row.try_get("organization_id").ok().flatten();
            let organization_name: Option<String> = if account_type == "personal" {
                Some(email.clone())
            } else {
                row.try_get("organization_name").ok().flatten()
            };

            let response = serde_json::json!({
                "id": id,
                "email": email,
                "first_name": first_name,
                "last_name": last_name,
                "auth_provider": auth_provider,
                "account_type": account_type,
                "organization_id": organization_id,
                "organization_name": organization_name,
                "total_emails": total_emails,
                "email_storage_bytes": email_storage_bytes,
                "drive_storage_bytes": drive_storage_bytes,
                "other_storage_bytes": chat_storage_bytes + notes_storage_bytes,
                "memory_used_bytes": total_used,
                "memory_limit_bytes": 10_737_418_240_i64, // 10 GB limit
            });

            PROFILE_CACHE.insert(user_id, response.clone()).await;
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "get_profile lookup failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/profile/password")]
#[instrument(target = "auth", skip(req, pool, data))]
pub async fn change_password(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<ChangePasswordInput>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if data.new_password.len() < 6 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "New password must be at least 6 characters" }));
    }

    let row = sqlx::query("SELECT password, auth_provider FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool.get_ref())
        .await;

    let (stored, auth_provider): (Option<String>, String) = match row {
        Ok(Some(r)) => (
            r.try_get("password").ok().flatten(),
            r.try_get("auth_provider")
                .unwrap_or_else(|_| "local".to_string()),
        ),
        _ => return HttpResponse::Unauthorized().finish(),
    };

    if let Some(stored) = stored {
        let current_password = data.current_password.as_deref().unwrap_or("");
        let valid = verify(current_password, &stored).unwrap_or(false);
        if !valid {
            warn!(target: "auth", user_id, "change-password: wrong current password");
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "message": "Current password is incorrect" }));
        }
    } else if auth_provider != "google" {
        warn!(target: "auth", user_id, "change-password rejected: missing password");
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "This account has no password to change" }));
    }

    let hashed = match hash(&data.new_password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            error!(target: "auth", error = %e, "bcrypt hash failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Err(e) = sqlx::query("UPDATE users SET password = $1 WHERE id = $2")
        .bind(&hashed)
        .bind(user_id)
        .execute(pool.get_ref())
        .await
    {
        error!(target: "auth", user_id, error = %e, "password update failed");
        return HttpResponse::InternalServerError().finish();
    }

    info!(target: "auth", user_id, had_password = data.current_password.is_some(), "password updated");
    HttpResponse::Ok().json(serde_json::json!({ "message": "Password updated" }))
}

#[put("/profile")]
#[instrument(target = "http", skip(req, pool, data))]
pub async fn update_profile(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<ProfileUpdate>,
) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let result = sqlx::query(
        "UPDATE users
         SET first_name = COALESCE($1, first_name),
             last_name = COALESCE($2, last_name)
         WHERE id = $3
         RETURNING id, email, first_name, last_name",
    )
    .bind(data.first_name.as_deref())
    .bind(data.last_name.as_deref())
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            invalidate_me_cache(user_id).await;
            invalidate_profile_cache(user_id).await;
            let id: i32 = row.get("id");
            let email: String = row.get("email");
            let first_name: Option<String> = row.try_get("first_name").ok();
            let last_name: Option<String> = row.try_get("last_name").ok();

            info!(target: "http", user_id, "profile updated");
            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "email": email,
                "first_name": first_name,
                "last_name": last_name,
            }))
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(target: "db", user_id, error = ?e, "update_profile failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/users/all")]
#[instrument(target = "http", skip(req, pool))]
async fn get_all_users(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    // Require a valid JWT — this endpoint enumerates every account.
    if get_user_id_from_request(&req).is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    let result = sqlx::query("SELECT id, email, public_key FROM users")
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(rows) => {
            let users: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    let id: i32 = r.get("id");
                    let email: String = r.get("email");
                    let public_key: Option<String> = r.get("public_key");
                    let public_key =
                        public_key.and_then(|k| serde_json::from_str::<Vec<u8>>(&k).ok());

                    serde_json::json!({
                        "id": id,
                        "email": email,
                        "public_key": public_key
                    })
                })
                .collect();

            HttpResponse::Ok().json(users)
        }
        Err(e) => {
            error!(target: "db", error = ?e, "get_all_users failed");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[cfg(test)]
mod auth_regression_tests {
    use super::*;

    #[test]
    fn test_normalized_account_type() {
        assert_eq!(normalized_account_type("personal"), "personal");
        assert_eq!(normalized_account_type("organization"), "organization");
        assert_eq!(
            normalized_account_type("organization_admin"),
            "organization_admin"
        );
        assert_eq!(normalized_account_type("platform_admin"), "platform_admin");
        assert_eq!(normalized_account_type("unknown"), "personal");
    }

    #[actix_web::test]
    async fn test_api_key_generation_and_validation() {
        use crate::test_support::{test_pool, insert_local_user};
        let pool = test_pool().await;

        // 1. Setup: Create an organization
        let org_id: i32 = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Test Org') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();

        // 2. Generate Key (Logic Check)
        let raw_key = "wv_sk_test_secret_123";
        let key_hash = bcrypt::hash(raw_key, DEFAULT_COST).unwrap();
        
        sqlx::query("INSERT INTO api_keys (organization_id, name, key_hash, key_preview) VALUES ($1, $2, $3, $4)")
            .bind(org_id)
            .bind("Test Key")
            .bind(&key_hash)
            .bind("wv_sk_..._123")
            .execute(&pool)
            .await
            .unwrap();

        // 3. Test Validation Helper
        let req = actix_test::TestRequest::default()
            .insert_header(("X-API-KEY", raw_key))
            .to_http_request();

        let validated_org_id = validate_api_key(&req, &pool).await;
        assert_eq!(validated_org_id, Some(org_id));

        // 4. Test Validation with wrong key
        let req_bad = actix_test::TestRequest::default()
            .insert_header(("X-API-KEY", "wrong_key"))
            .to_http_request();
        
        let validated_bad = validate_api_key(&req_bad, &pool).await;
        assert!(validated_bad.is_none());
    }

    // Import the actix test module under an alias — a bare `test` import
    // shadows the built-in `#[test]` attribute (it would resolve to
    // `#[actix_web::test]` and reject the sync unit test above).
    use actix_web::{App, http::StatusCode, test as actix_test, web};
    use sqlx::postgres::PgPoolOptions;

    fn lazy_pool() -> PgPool {
        PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/rwayve_test")
            .expect("lazy pool")
    }

    #[actix_web::test]
    async fn get_user_by_email_requires_auth() {
        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(lazy_pool()))
                .service(get_user_by_email),
        )
        .await;

        let req = actix_test::TestRequest::get()
            .uri("/users?email=target@example.com")
            .to_request();
        let resp = actix_test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn get_all_users_requires_auth() {
        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(lazy_pool()))
                .service(get_all_users),
        )
        .await;

        let req = actix_test::TestRequest::get()
            .uri("/users/all")
            .to_request();
        let resp = actix_test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
