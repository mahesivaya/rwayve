#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{insert_local_user, jwt_for, random_email, test_pool};
    use actix_web::{App, http::StatusCode, test, web};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn set_env(key: &str, val: &str) {
        unsafe {
            std::env::set_var(key, val);
        }
    }

    fn set_zoom_creds() {
        set_env("ZOOM_ACCOUNT_ID", "acc-1");
        set_env("ZOOM_CLIENT_ID", "cid");
        set_env("ZOOM_CLIENT_SECRET", "csec");
    }

    /// Tomorrow's date so the "no past meetings" check passes.
    fn tomorrow_date_str() -> String {
        (chrono::Utc::now().date_naive() + chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string()
    }

    #[actix_web::test]
    async fn meetings_endpoints_require_auth() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .service(create_meeting)
                .service(get_meetings)
                .service(update_meeting)
                .service(delete_meeting),
        )
        .await;

        let resp = test::call_service(
            &app,
            test::TestRequest::get().uri("/meetings").to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let resp = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/meetings")
                .set_json(serde_json::json!({}))
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let resp = test::call_service(
            &app,
            test::TestRequest::delete().uri("/meetings/1").to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn create_meeting_full_fanout_persists_meeting_and_calls_zoom_and_gmail() {
        let server = MockServer::start().await;
        set_zoom_creds();
        set_env("ZOOM_OAUTH_TOKEN_URL", &format!("{}/oauth/token", server.uri()));
        set_env("ZOOM_API_BASE", &server.uri());
        set_env("GMAIL_SEND_URL", &format!("{}/gmail/send", server.uri()));

        // Zoom OAuth + meeting create
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "zoom-tok",
                "expires_in": 3600,
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v2/users/me/meetings"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 99,
                "join_url": "https://zoom.example/j/99",
            })))
            .mount(&server)
            .await;

        // Gmail send (the spawned invite email will hit this)
        let gmail_mock = Mock::given(method("POST"))
            .and(path("/gmail/send"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "gmail-msg-1"
            })))
            .expect(1..)
            .named("gmail send")
            .mount_as_scoped(&server)
            .await;

        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "p").await;

        // Active gmail account is required for invite email to fire.
        sqlx::query(
            "INSERT INTO email_accounts (email, user_id, access_token, refresh_token, token_expiry, is_active)
             VALUES ($1,$2,$3,$4, NOW() + INTERVAL '1 hour', true)",
        )
        .bind(&email)
        .bind(user_id)
        .bind("active-gmail-tok")
        .bind("rt")
        .execute(&pool)
        .await
        .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(create_meeting),
        )
        .await;

        let payload = serde_json::json!({
            "title": "Standup",
            "date": tomorrow_date_str(),
            "start": 9 * 60,         // 09:00
            "end": 9 * 60 + 30,      // 09:30
            "participants": [format!("invitee-{}@example.com", uuid::Uuid::new_v4())],
            "tz": "UTC",
        });

        let req = test::TestRequest::post()
            .uri("/meetings")
            .insert_header(("Authorization", format!("Bearer {}", jwt_for(user_id, &email))))
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let meeting_id = body["meeting_id"].as_i64().expect("meeting_id");

        // Meeting row exists with the Zoom URL we mocked.
        let row = sqlx::query("SELECT title, zoom_join_url FROM meetings WHERE id = $1")
            .bind(meeting_id as i32)
            .fetch_one(&pool)
            .await
            .unwrap();
        let title: String = sqlx::Row::get(&row, "title");
        let join_url: Option<String> = sqlx::Row::try_get(&row, "zoom_join_url").unwrap_or(None);
        assert_eq!(title, "Standup");
        assert_eq!(join_url.as_deref(), Some("https://zoom.example/j/99"));

        // Participants row was inserted.
        let p_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM meeting_participants WHERE meeting_id = $1",
        )
        .bind(meeting_id as i32)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(p_count, 1);

        // Wait briefly for the spawned invite email to fire, then drop the
        // scoped Mock — its Drop checks `expect(1..)` and panics if it
        // didn't see at least one request.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        drop(gmail_mock);

        sqlx::query("DELETE FROM meeting_participants WHERE meeting_id = $1")
            .bind(meeting_id as i32)
            .execute(&pool)
            .await
            .ok();
        sqlx::query("DELETE FROM meetings WHERE id = $1")
            .bind(meeting_id as i32)
            .execute(&pool)
            .await
            .ok();
        sqlx::query("DELETE FROM email_accounts WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        crate::test_support::delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn create_meeting_continues_when_zoom_fails() {
        let server = MockServer::start().await;
        set_zoom_creds();
        set_env("ZOOM_OAUTH_TOKEN_URL", &format!("{}/oauth/token", server.uri()));
        set_env("ZOOM_API_BASE", &server.uri());
        set_env("GMAIL_SEND_URL", &format!("{}/gmail/send", server.uri()));

        // Zoom token fails — meeting should still be created without join URL.
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(500).set_body_string("zoom down"))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/gmail/send"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "p").await;

        sqlx::query(
            "INSERT INTO email_accounts (email, user_id, access_token, refresh_token, token_expiry, is_active)
             VALUES ($1,$2,$3,$4, NOW() + INTERVAL '1 hour', true)",
        )
        .bind(&email)
        .bind(user_id)
        .bind("tok")
        .bind("rt")
        .execute(&pool)
        .await
        .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(create_meeting),
        )
        .await;

        let payload = serde_json::json!({
            "title": "ZoomDown",
            "date": tomorrow_date_str(),
            "start": 10 * 60,
            "end": 10 * 60 + 30,
            "participants": ["alone@example.com"],
            "tz": "UTC",
        });

        let req = test::TestRequest::post()
            .uri("/meetings")
            .insert_header(("Authorization", format!("Bearer {}", jwt_for(user_id, &email))))
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let meeting_id = body["meeting_id"].as_i64().unwrap() as i32;

        let join: Option<String> =
            sqlx::query_scalar("SELECT zoom_join_url FROM meetings WHERE id = $1")
                .bind(meeting_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(join.is_none(), "Zoom failure must leave join_url NULL");

        sqlx::query("DELETE FROM meeting_participants WHERE meeting_id = $1")
            .bind(meeting_id)
            .execute(&pool)
            .await
            .ok();
        sqlx::query("DELETE FROM meetings WHERE id = $1")
            .bind(meeting_id)
            .execute(&pool)
            .await
            .ok();
        sqlx::query("DELETE FROM email_accounts WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
        crate::test_support::delete_user(&pool, user_id).await;
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn create_meeting_rejects_past_date() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "p").await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(create_meeting),
        )
        .await;

        let yesterday = (chrono::Utc::now().date_naive() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        let payload = serde_json::json!({
            "title": "Past",
            "date": yesterday,
            "start": 9 * 60,
            "end": 10 * 60,
            "participants": ["x@example.com"],
            "tz": "UTC",
        });

        let req = test::TestRequest::post()
            .uri("/meetings")
            .insert_header(("Authorization", format!("Bearer {}", jwt_for(user_id, &email))))
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        crate::test_support::delete_user(&pool, user_id).await;
    }
}
