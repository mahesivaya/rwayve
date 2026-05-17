#[cfg(test)]
mod tests {
    use crate::scheduler::zoom::create_zoom_meeting;
    use chrono::Utc;
    use serde_json::json;
    use wiremock::matchers::{header_regex, method, path};
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

    #[tokio::test]
    #[serial_test::serial]
    async fn create_zoom_meeting_returns_join_url_on_success() {
        let server = MockServer::start().await;
        set_zoom_creds();
        set_env(
            "ZOOM_OAUTH_TOKEN_URL",
            &format!("{}/oauth/token", server.uri()),
        );
        set_env("ZOOM_API_BASE", &server.uri());

        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(header_regex("Authorization", "^Basic "))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "zoom-tok",
                "token_type": "bearer",
                "expires_in": 3600,
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v2/users/me/meetings"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": 12345,
                "join_url": "https://zoom.example/j/12345",
            })))
            .mount(&server)
            .await;

        let when = Utc::now() + chrono::Duration::hours(1);
        let url = create_zoom_meeting("Standup", when, 30).await.unwrap();
        assert_eq!(url, "https://zoom.example/j/12345");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn create_zoom_meeting_propagates_token_failure() {
        let server = MockServer::start().await;
        set_zoom_creds();
        set_env(
            "ZOOM_OAUTH_TOKEN_URL",
            &format!("{}/oauth/token", server.uri()),
        );
        set_env("ZOOM_API_BASE", &server.uri());

        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(401).set_body_string("invalid_client"))
            .mount(&server)
            .await;

        let res = create_zoom_meeting("Standup", Utc::now() + chrono::Duration::hours(1), 30).await;
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Zoom token error"), "got: {err}");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn create_zoom_meeting_propagates_create_failure() {
        let server = MockServer::start().await;
        set_zoom_creds();
        set_env(
            "ZOOM_OAUTH_TOKEN_URL",
            &format!("{}/oauth/token", server.uri()),
        );
        set_env("ZOOM_API_BASE", &server.uri());

        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "zoom-tok",
                "expires_in": 3600,
            })))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v2/users/me/meetings"))
            .respond_with(ResponseTemplate::new(429).set_body_string("rate limited"))
            .mount(&server)
            .await;

        let res = create_zoom_meeting("Standup", Utc::now() + chrono::Duration::hours(1), 30).await;
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Zoom create error"), "got: {err}");
    }
}
