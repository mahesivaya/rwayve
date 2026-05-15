#[cfg(test)]
mod tests {
    use crate::routes::auth::{forgot_password, login, register, reset_password};
    use crate::test_support::{
        delete_user, insert_google_user, insert_local_user, random_email, test_pool,
    };
    use actix_web::{App, http::StatusCode, test, web};
    use bcrypt::verify;
    use serde_json::json;

    fn body_message(body: &[u8]) -> String {
        let v: serde_json::Value = serde_json::from_slice(body).unwrap_or(json!({}));
        v.get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string()
    }

    #[actix_web::test]
    async fn register_creates_user_and_returns_token() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(register),
        )
        .await;

        let email = random_email();
        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(json!({
                "email": email,
                "password": "secret123",
                "confirm_password": "secret123",
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("token").is_some(), "expected token in response");

        sqlx::query("DELETE FROM users WHERE email = $1")
            .bind(&email)
            .execute(&pool)
            .await
            .ok();
    }

    #[actix_web::test]
    async fn register_rejects_password_mismatch() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(register),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(json!({
                "email": random_email(),
                "password": "abc123",
                "confirm_password": "different",
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = test::read_body(resp).await;
        assert!(body_message(&body).contains("not match"));
    }

    #[actix_web::test]
    async fn register_rejects_duplicate_email() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "first").await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(register),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/register")
            .set_json(json!({
                "email": email,
                "password": "second123",
                "confirm_password": "second123",
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = test::read_body(resp).await;
        assert!(body_message(&body).contains("already exists"));

        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn login_succeeds_with_correct_password() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "rightpass").await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(login),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(json!({ "email": email, "password": "rightpass" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.get("token").is_some());

        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn login_rejects_wrong_password() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "rightpass").await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(login),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(json!({ "email": email, "password": "WRONG" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn login_rejects_unknown_user() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(login),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(json!({ "email": "nobody@nowhere.test", "password": "x" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn login_rejects_google_user_with_password() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_google_user(&pool, &email).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(login),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(json!({ "email": email, "password": "anything" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let body = test::read_body(resp).await;
        assert!(
            body_message(&body).contains("Google"),
            "should hint at Google sign-in"
        );

        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn forgot_password_returns_generic_message_for_unknown_email() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(forgot_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/forgot-password")
            .set_json(json!({ "email": "definitely-not-a-user@test.local" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn forgot_password_creates_token_for_local_user() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "secret").await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(forgot_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/forgot-password")
            .set_json(json!({ "email": email }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM password_reset_tokens WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(count >= 1, "expected at least one reset token row");

        sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn forgot_password_skips_google_users() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_google_user(&pool, &email).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(forgot_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/forgot-password")
            .set_json(json!({ "email": email }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM password_reset_tokens WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 0, "Google users must not get reset tokens");

        delete_user(&pool, user_id).await;
    }

    async fn create_reset_token(pool: &sqlx::PgPool, user_id: i32, ttl_minutes: i64) -> String {
        let token = format!("test-token-{}", uuid::Uuid::new_v4());
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(ttl_minutes);
        sqlx::query(
            "INSERT INTO password_reset_tokens (user_id, token, expires_at) VALUES ($1,$2,$3)",
        )
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(pool)
        .await
        .unwrap();
        token
    }

    #[actix_web::test]
    async fn reset_password_with_valid_token_updates_hash_and_marks_used() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "old-pw").await;
        let token = create_reset_token(&pool, user_id, 30).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(reset_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/reset-password")
            .set_json(json!({ "token": token, "new_password": "brand-new-pw" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // The new password must verify; the old must not.
        let row = sqlx::query("SELECT password FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let stored: Option<String> = sqlx::Row::try_get(&row, "password").unwrap_or(None);
        let stored = stored.expect("password should be set");
        assert!(verify("brand-new-pw", &stored).unwrap());
        assert!(!verify("old-pw", &stored).unwrap());

        // Token should now be marked used.
        let used: Option<chrono::DateTime<chrono::Utc>> =
            sqlx::query_scalar("SELECT used_at FROM password_reset_tokens WHERE token = $1")
                .bind(&token)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(used.is_some(), "used_at must be populated");

        sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn reset_password_rejects_expired_token() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "old-pw").await;
        let token = create_reset_token(&pool, user_id, -10).await; // already expired

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(reset_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/reset-password")
            .set_json(json!({ "token": token, "new_password": "whatever" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn reset_password_rejects_unknown_token() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(reset_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/reset-password")
            .set_json(json!({ "token": "no-such-token", "new_password": "whatever" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn reset_password_rejects_short_password() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "old-pw").await;
        let token = create_reset_token(&pool, user_id, 30).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(reset_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/reset-password")
            .set_json(json!({ "token": token, "new_password": "short" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    async fn reset_password_rejects_already_used_token() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "old-pw").await;
        let token = create_reset_token(&pool, user_id, 30).await;

        // Mark it used up front.
        sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE token = $1")
            .bind(&token)
            .execute(&pool)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(reset_password),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/reset-password")
            .set_json(json!({ "token": token, "new_password": "fresh-pass" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        delete_user(&pool, user_id).await;
    }
}
