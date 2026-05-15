#[cfg(test)]
mod tests {
    use crate::email::handler::{get_me, oauth_callback, save_public_key, send};
    use crate::test_support::{
        delete_user, insert_google_user, insert_local_user, jwt_for, random_email, test_pool,
    };
    use actix_web::{App, http::StatusCode, test, web};
    use std::io::Write;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Write a stub client_secret.json into a temp dir and return its path.
    /// Tests then set GOOGLE_CLIENT_SECRET_PATH to point at it. We use
    /// a unique suffix per call so parallel tests don't clobber each other.
    fn write_fake_client_secret() -> String {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "rwayve-client-secret-{}.json",
            uuid::Uuid::new_v4()
        ));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(
            br#"{"web":{"client_id":"test-client","client_secret":"test-secret","redirect_uris":["http://test.local/oauth/callback"]}}"#,
        )
        .unwrap();
        path.to_string_lossy().to_string()
    }

    fn set_env(key: &str, val: &str) {
        // SAFETY: tests that mutate env are #[serial_test::serial] so they
        // don't race with each other across the process.
        unsafe {
            std::env::set_var(key, val);
        }
    }

    /// Seed a single-use signup OAuth state row and return its opaque token.
    /// `oauth_callback` validates `state` against the `oauth_states` table,
    /// so tests must store one rather than passing a literal marker.
    async fn seed_signup_state(pool: &sqlx::PgPool) -> String {
        let state = format!("test-state-{}", uuid::Uuid::new_v4());
        crate::security::oauth::store_state(&state, None, "signup", pool)
            .await
            .expect("store signup oauth state");
        state
    }

    /// Configure all the env vars OAuth callback needs to point at the mock,
    /// returning the FRONTEND_URL so callers can build expected redirect URLs.
    async fn setup_google_mocks(server: &MockServer, email: &str) -> String {
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "fake-access",
                "refresh_token": "fake-refresh",
                "expires_in": 3600,
            })))
            .mount(server)
            .await;

        Mock::given(method("GET"))
            .and(path("/userinfo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "email": email,
            })))
            .mount(server)
            .await;

        // Calendar import is best-effort; return an empty event list.
        Mock::given(method("GET"))
            .and(path("/calendar"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": []
            })))
            .mount(server)
            .await;

        let secret_path = write_fake_client_secret();
        set_env("GOOGLE_CLIENT_SECRET_PATH", &secret_path);
        set_env("GOOGLE_TOKEN_URL", &format!("{}/token", server.uri()));
        set_env("GOOGLE_USERINFO_URL", &format!("{}/userinfo", server.uri()));
        set_env("GOOGLE_CALENDAR_URL", &format!("{}/calendar", server.uri()));
        let frontend = "http://test-frontend.local";
        set_env("FRONTEND_URL", frontend);
        frontend.to_string()
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn oauth_callback_signup_creates_new_user_and_redirects_home() {
        let server = MockServer::start().await;
        let google_email = random_email();
        let frontend = setup_google_mocks(&server, &google_email).await;

        let pool = test_pool().await;
        let state = seed_signup_state(&pool).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/oauth/callback", web::get().to(oauth_callback)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/oauth/callback?code=anything&state={state}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);

        let location = resp
            .headers()
            .get("Location")
            .expect("Location header set")
            .to_str()
            .unwrap()
            .to_string();
        // Signup redirects to the SPA landing route with a `#signup=true`
        // fragment; the session token is set as an httpOnly cookie, not in
        // the URL.
        assert_eq!(
            location,
            format!("{frontend}/home#signup=true"),
            "unexpected redirect: {location}"
        );

        // The user must now exist with auth_provider='google' and a NULL password.
        let row = sqlx::query("SELECT id, password, auth_provider FROM users WHERE email = $1")
            .bind(&google_email)
            .fetch_one(&pool)
            .await
            .expect("user inserted");
        let user_id: i32 = sqlx::Row::get(&row, "id");
        let pw: Option<String> = sqlx::Row::try_get(&row, "password").unwrap_or(None);
        let provider: String = sqlx::Row::get(&row, "auth_provider");
        assert!(pw.is_none(), "google user must have NULL password");
        assert_eq!(provider, "google");

        // email_accounts row must also be created so sync can begin.
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM email_accounts WHERE user_id = $1 AND email = $2",
        )
        .bind(user_id)
        .bind(&google_email)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1);

        sqlx::query("DELETE FROM email_accounts WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn oauth_callback_signup_blocks_local_email_collision() {
        let server = MockServer::start().await;
        let collision_email = random_email();
        let frontend = setup_google_mocks(&server, &collision_email).await;

        // Pre-existing local user with the same email.
        let pool = test_pool().await;
        let state = seed_signup_state(&pool).await;
        let user_id = insert_local_user(&pool, &collision_email, "local-pw").await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/oauth/callback", web::get().to(oauth_callback)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/oauth/callback?code=anything&state={state}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);

        let location = resp
            .headers()
            .get("Location")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert_eq!(
            location,
            format!("{frontend}/login?error=email_exists"),
            "must redirect back to login with the collision marker"
        );

        // No new google account row should have been created.
        let google_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE email = $1 AND auth_provider = 'google'",
        )
        .bind(&collision_email)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(google_count, 0);

        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn oauth_callback_signup_signs_in_existing_google_user() {
        let server = MockServer::start().await;
        let email = random_email();
        let frontend = setup_google_mocks(&server, &email).await;

        let pool = test_pool().await;
        let state = seed_signup_state(&pool).await;
        let existing_user_id = insert_google_user(&pool, &email).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/oauth/callback", web::get().to(oauth_callback)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/oauth/callback?code=anything&state={state}"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert_eq!(location, format!("{frontend}/home#signup=true"));

        // No duplicate user must have been inserted.
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
            .bind(&email)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "should reuse existing google user");

        sqlx::query("DELETE FROM email_accounts WHERE user_id = $1")
            .bind(existing_user_id)
            .execute(&pool)
            .await
            .ok();
        delete_user(&pool, existing_user_id).await;
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn send_email_succeeds_with_mocked_gmail() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/gmail/send"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg-1"
            })))
            .mount(&server)
            .await;
        set_env("GMAIL_SEND_URL", &format!("{}/gmail/send", server.uri()));

        // `send` refreshes the OAuth access token before calling Gmail.
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "fresh-access",
                "expires_in": 3600,
            })))
            .mount(&server)
            .await;
        set_env("GOOGLE_TOKEN_URL", &format!("{}/token", server.uri()));
        set_env("GOOGLE_CLIENT_SECRET_PATH", &write_fake_client_secret());

        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "p").await;

        // Create a Gmail account row this user can send from.
        let account_row = sqlx::query(
            "INSERT INTO email_accounts (email, user_id, access_token, refresh_token, token_expiry, is_active)
             VALUES ($1,$2,$3,$4, NOW() + INTERVAL '1 hour', true)
             RETURNING id",
        )
        .bind(&email)
        .bind(user_id)
        .bind("fake-access")
        .bind("fake-refresh")
        .fetch_one(&pool)
        .await
        .unwrap();
        let account_id: i32 = sqlx::Row::get(&account_row, "id");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(send),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/send")
            .insert_header((
                "Authorization",
                format!("Bearer {}", jwt_for(user_id, &email)),
            ))
            .set_json(serde_json::json!({
                "account_id": account_id,
                "to": "recipient@example.com",
                "subject": "hello",
                "body": "world",
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        sqlx::query("DELETE FROM email_accounts WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn send_email_returns_500_when_gmail_rejects() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/gmail/send"))
            .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
            .mount(&server)
            .await;
        set_env("GMAIL_SEND_URL", &format!("{}/gmail/send", server.uri()));

        // `send` refreshes the OAuth access token before calling Gmail.
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "fresh-access",
                "expires_in": 3600,
            })))
            .mount(&server)
            .await;
        set_env("GOOGLE_TOKEN_URL", &format!("{}/token", server.uri()));
        set_env("GOOGLE_CLIENT_SECRET_PATH", &write_fake_client_secret());

        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "p").await;
        let account_row = sqlx::query(
            "INSERT INTO email_accounts (email, user_id, access_token, refresh_token, token_expiry, is_active)
             VALUES ($1,$2,$3,$4, NOW() + INTERVAL '1 hour', true)
             RETURNING id",
        )
        .bind(&email)
        .bind(user_id)
        .bind("fake")
        .bind("fake")
        .fetch_one(&pool)
        .await
        .unwrap();
        let account_id: i32 = sqlx::Row::get(&account_row, "id");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(send),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/send")
            .insert_header((
                "Authorization",
                format!("Bearer {}", jwt_for(user_id, &email)),
            ))
            .set_json(serde_json::json!({
                "account_id": account_id,
                "to": "x@y.z",
                "subject": "s",
                "body": "b",
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        sqlx::query("DELETE FROM email_accounts WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn send_requires_auth() {
        let pool = test_pool().await;
        let app = test::init_service(App::new().app_data(web::Data::new(pool)).service(send)).await;
        let req = test::TestRequest::post()
            .uri("/send")
            .set_json(serde_json::json!({
                "account_id": 1,
                "to": "x@y.z",
                "subject": "hi",
                "body": "test"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn me_requires_auth() {
        let pool = test_pool().await;
        let app =
            test::init_service(App::new().app_data(web::Data::new(pool)).service(get_me)).await;
        let req = test::TestRequest::get().uri("/me").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn save_public_key_requires_auth() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .service(save_public_key),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/save-public-key")
            .set_json(serde_json::json!({ "public_key": [1,2,3] }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
