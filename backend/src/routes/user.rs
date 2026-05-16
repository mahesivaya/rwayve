use crate::models::auth::ChangePasswordInput;
use crate::models::email_request::UserResponse;
use crate::security::jwt::get_user_id_from_request;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, put, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{error, info, instrument, warn};

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
pub struct AdminCreateUserInput {
    /// Email local part. The full address is generated server-side as
    /// <handle>@<domain> — the business slug domain, or wayve.com for personal.
    pub handle: String,
    pub password: String,
    pub account_type: Option<String>,
    pub organization_name: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateOrganizationInput {
    pub name: String,
    /// Optional business admin to provision with the business. The handle is
    /// the email local part; the address becomes <handle>@<business-slug>.com.
    /// If either field is supplied, both are required.
    pub admin_handle: Option<String>,
    pub admin_password: Option<String>,
}

fn normalized_account_type(value: &str) -> &str {
    match value {
        "business" => "business",
        "business_admin" => "business_admin",
        "project_admin" => "project_admin",
        _ => "personal",
    }
}

/// Slug for a business: lowercase, ASCII-alphanumeric only. Mirrors the SQL
/// backfill in init.sql. "Acme Corp" -> "acmecorp", "One" -> "one".
fn slugify(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

/// Email domain for an account: the business slug as `<slug>.com`, or
/// `wayve.com` for accounts that don't belong to a business.
fn email_domain_for(org_slug: Option<&str>) -> String {
    match org_slug {
        Some(slug) if !slug.is_empty() => format!("{slug}.com"),
        _ => "wayve.com".to_string(),
    }
}

async fn require_project_admin(req: &HttpRequest, pool: &PgPool) -> Result<i32, HttpResponse> {
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
                error!(target: "db", admin_id, error = ?e, "project admin lookup failed");
                return Err(HttpResponse::InternalServerError().finish());
            }
        };

    if normalized_account_type(account_type.as_deref().unwrap_or("personal")) != "project_admin" {
        return Err(HttpResponse::Forbidden()
            .json(serde_json::json!({ "message": "Only project admins can manage businesses" })));
    }

    Ok(admin_id)
}

#[get("/admin/organizations")]
#[instrument(target = "http", skip(req, pool))]
pub async fn admin_list_organizations(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    if let Err(response) = require_project_admin(&req, pool.get_ref()).await {
        return response;
    }

    match sqlx::query(
        r#"
        SELECT
            o.id,
            o.name,
            o.slug,
            o.created_at,
            COUNT(u.id) AS user_count
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
                    let slug: Option<String> = row.try_get("slug").ok().flatten();
                    let user_count: i64 = row.get("user_count");

                    serde_json::json!({
                        "id": id,
                        "name": name,
                        "slug": slug,
                        "user_count": user_count
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
    let admin_id = match require_project_admin(&req, pool.get_ref()).await {
        Ok(id) => id,
        Err(response) => return response,
    };

    let name = data.name.trim();
    if name.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "Business name is required" }));
    }

    let slug = slugify(name);
    if slug.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Business name must contain letters or numbers"
        }));
    }

    // The business admin block is optional, but if either field is supplied
    // both are required. The handle becomes <handle>@<business-slug>.com.
    let raw_handle = data
        .admin_handle
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    let raw_password = data.admin_password.as_deref().unwrap_or("");

    let business_admin = if !raw_handle.is_empty() || !raw_password.is_empty() {
        let handle = slugify(raw_handle);
        if handle.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Business admin handle must contain letters or numbers"
            }));
        }
        if raw_password.len() < 6 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Password must be at least 6 characters"
            }));
        }
        Some((handle, raw_password.to_string()))
    } else {
        None
    };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(target: "db", admin_id, error = ?e, "begin business transaction failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let org_row = match sqlx::query(
        r#"
        INSERT INTO organizations (name, slug)
        VALUES ($1, $2)
        ON CONFLICT (name) DO UPDATE SET slug = EXCLUDED.slug
        RETURNING id, name, slug
        "#,
    )
    .bind(name)
    .bind(&slug)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "message": "Another business already uses that name slug"
                }));
            }
            error!(target: "db", admin_id, error = ?e, "create organization failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let organization_id: i32 = org_row.get("id");
    let organization_name: String = org_row.get("name");
    let organization_slug: String = org_row.get("slug");

    let mut admin_json = serde_json::Value::Null;

    if let Some((handle, password)) = business_admin {
        let email = format!("{handle}@{organization_slug}.com");
        let hashed = match hash(&password, DEFAULT_COST) {
            Ok(value) => value,
            Err(e) => {
                error!(target: "auth", error = %e, "business admin bcrypt hash failed");
                return HttpResponse::InternalServerError().finish();
            }
        };

        // username == email keeps the global username-unique index satisfied.
        match sqlx::query(
            r#"
            INSERT INTO users (username, email, password, auth_provider, account_type, organization_id)
            VALUES ($1, $1, $2, 'local', 'business_admin', $3)
            RETURNING id, username, email, account_type, organization_id
            "#,
        )
        .bind(&email)
        .bind(&hashed)
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
                    "account_type": normalized_account_type(&account_type),
                    "organization_id": org_id
                });
            }
            Err(e) => {
                if e.to_string().contains("duplicate key") {
                    return HttpResponse::Conflict().json(serde_json::json!({
                        "message": "A business admin with that email already exists"
                    }));
                }
                error!(target: "db", admin_id, error = ?e, "create business admin failed");
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

    if let Err(e) = tx.commit().await {
        error!(target: "db", admin_id, error = ?e, "commit business transaction failed");
        return HttpResponse::InternalServerError().finish();
    }

    let user_count = if admin_json.is_null() { 0 } else { 1 };
    info!(target: "auth", admin_id, organization_id, "project admin created business");
    HttpResponse::Created().json(serde_json::json!({
        "id": organization_id,
        "name": organization_name,
        "slug": organization_slug,
        "user_count": user_count,
        "admin": admin_json
    }))
}

#[post("/admin/users")]
#[instrument(target = "auth", skip(req, pool, data), fields(handle = %data.handle))]
pub async fn admin_create_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    data: web::Json<AdminCreateUserInput>,
) -> impl Responder {
    let admin_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let admin_row = match sqlx::query(
        r#"
        SELECT u.account_type, u.organization_id, o.slug AS org_slug
        FROM users u
        LEFT JOIN organizations o ON o.id = u.organization_id
        WHERE u.id = $1
        "#,
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

    let Some(admin_row) = admin_row else {
        return HttpResponse::Unauthorized().finish();
    };

    let admin_account_type: String = admin_row
        .try_get("account_type")
        .unwrap_or_else(|_| "personal".to_string());
    let admin_account_type = normalized_account_type(&admin_account_type);
    let admin_organization_id: Option<i32> = admin_row.try_get("organization_id").ok().flatten();
    let admin_org_slug: Option<String> = admin_row.try_get("org_slug").ok().flatten();

    if !matches!(admin_account_type, "business_admin" | "project_admin") {
        warn!(target: "auth", admin_id, "non-business user tried to create user");
        return HttpResponse::Forbidden()
            .json(serde_json::json!({ "message": "Only admins can create users" }));
    }

    let handle = slugify(&data.handle);
    if handle.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "A handle (letters or numbers) is required"
        }));
    }
    if data.password.len() < 6 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "Password must be at least 6 characters" }));
    }

    let requested_account_type = data
        .account_type
        .as_deref()
        .map(normalized_account_type)
        .unwrap_or("personal");

    let account_type = match admin_account_type {
        "project_admin" => match requested_account_type {
            "business_admin" | "project_admin" | "business" | "personal" => requested_account_type,
            _ => "personal",
        },
        "business_admin" => "business",
        _ => "personal",
    };

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!(target: "db", admin_id, error = ?e, "begin create-user transaction failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Resolve which business the new account belongs to. The org slug decides
    // the email domain: <handle>@<slug>.com, or @wayve.com when there is none.
    let (organization_id, org_slug): (Option<i32>, Option<String>) =
        if account_type == "business_admin" {
            // project_admin provisioning a business admin for a named business.
            let organization_name = data
                .organization_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());

            let Some(organization_name) = organization_name else {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Organization name is required for business admin accounts"
                }));
            };

            let new_slug = slugify(organization_name);
            if new_slug.is_empty() {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Organization name must contain letters or numbers"
                }));
            }

            match sqlx::query(
                r#"
                INSERT INTO organizations (name, slug)
                VALUES ($1, $2)
                ON CONFLICT (name) DO UPDATE SET slug = EXCLUDED.slug
                RETURNING id, slug
                "#,
            )
            .bind(organization_name)
            .bind(&new_slug)
            .fetch_one(&mut *tx)
            .await
            {
                Ok(row) => (Some(row.get("id")), Some(row.get("slug"))),
                Err(e) => {
                    if e.to_string().contains("duplicate key") {
                        return HttpResponse::Conflict().json(serde_json::json!({
                            "message": "Another business already uses that name slug"
                        }));
                    }
                    error!(target: "db", admin_id, error = ?e, "organization upsert failed");
                    return HttpResponse::InternalServerError().finish();
                }
            }
        } else if admin_account_type == "business_admin" {
            // business_admin creating an account inside their own business.
            match admin_organization_id {
                Some(id) => (Some(id), admin_org_slug),
                None => {
                    return HttpResponse::BadRequest().json(serde_json::json!({
                        "message": "Business admin is not assigned to an organization"
                    }));
                }
            }
        } else {
            // project_admin creating a personal or project_admin account.
            (None, None)
        };

    let email = format!("{}@{}", handle, email_domain_for(org_slug.as_deref()));

    let hashed = match hash(&data.password, DEFAULT_COST) {
        Ok(value) => value,
        Err(e) => {
            error!(target: "auth", error = %e, "admin create user bcrypt hash failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Fold organization_id into the INSERT: the account lands in its business
    // atomically, or not at all. username == email satisfies the unique index.
    let row = match sqlx::query(
        r#"
        INSERT INTO users (username, email, password, auth_provider, account_type, organization_id)
        VALUES ($1, $1, $2, 'local', $3, $4)
        RETURNING id, username, email, account_type, organization_id
        "#,
    )
    .bind(&email)
    .bind(&hashed)
    .bind(account_type)
    .bind(organization_id)
    .fetch_one(&mut *tx)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "message": "An account with that email already exists"
                }));
            }
            error!(target: "db", admin_id, error = ?e, "admin create user failed");
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let Err(e) = tx.commit().await {
        error!(target: "db", admin_id, error = ?e, "commit create-user transaction failed");
        return HttpResponse::InternalServerError().finish();
    }

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
        "account_type": normalized_account_type(&account_type),
        "organization_id": organization_id
    }))
}

#[get("/profile")]
#[instrument(target = "http", skip(req, pool))]
pub async fn get_profile(req: HttpRequest, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = match get_user_id_from_request(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let result = sqlx::query(
        "SELECT id, email, first_name, last_name, auth_provider FROM users WHERE id = $1",
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
            let auth_provider: String = row.get("auth_provider");

            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "email": email,
                "first_name": first_name,
                "last_name": last_name,
                "auth_provider": auth_provider,
            }))
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
    use actix_web::{App, http::StatusCode, test, web};
    use sqlx::postgres::PgPoolOptions;

    fn lazy_pool() -> PgPool {
        PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/rwayve_test")
            .expect("lazy pool")
    }

    #[actix_web::test]
    async fn get_user_by_email_requires_auth() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(lazy_pool()))
                .service(get_user_by_email),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/users?email=target@example.com")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn get_all_users_requires_auth() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(lazy_pool()))
                .service(get_all_users),
        )
        .await;

        let req = test::TestRequest::get().uri("/users/all").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
