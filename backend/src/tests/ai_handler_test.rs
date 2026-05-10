#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::jwt_for;
    use actix_web::{App, http::StatusCode, test};
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn set_env(key: &str, val: &str) {
        unsafe {
            std::env::set_var(key, val);
        }
    }

    #[actix_web::test]
    async fn ai_chat_requires_auth() {
        let app = test::init_service(App::new().service(ai_chat)).await;
        let req = test::TestRequest::post()
            .uri("/ai/chat")
            .set_json(serde_json::json!({ "messages": [] }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn ai_chat_returns_500_when_api_key_missing() {
        unsafe {
            std::env::remove_var("GEMINI_API_KEY");
        }
        let app = test::init_service(App::new().service(ai_chat)).await;
        let req = test::TestRequest::post()
            .uri("/ai/chat")
            .insert_header(("Authorization", format!("Bearer {}", jwt_for(1, "x@y.z"))))
            .set_json(serde_json::json!({
                "messages": [{ "role": "user", "content": "hi" }]
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn ai_chat_returns_400_for_empty_conversation() {
        set_env("GEMINI_API_KEY", "fake-key");
        let app = test::init_service(App::new().service(ai_chat)).await;
        let req = test::TestRequest::post()
            .uri("/ai/chat")
            .insert_header(("Authorization", format!("Bearer {}", jwt_for(1, "x@y.z"))))
            .set_json(serde_json::json!({ "messages": [] }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn ai_chat_proxies_to_mocked_gemini_and_returns_reply() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "candidates": [{
                    "content": {
                        "parts": [{ "text": "hello from mock" }]
                    }
                }]
            })))
            .mount(&server)
            .await;

        set_env("GEMINI_API_KEY", "fake-key");
        set_env("GEMINI_API_BASE", &server.uri());
        set_env("GEMINI_MODEL", "gemini-2.0-flash");

        let app = test::init_service(App::new().service(ai_chat)).await;

        let req = test::TestRequest::post()
            .uri("/ai/chat")
            .insert_header(("Authorization", format!("Bearer {}", jwt_for(1, "x@y.z"))))
            .set_json(serde_json::json!({
                "messages": [{ "role": "user", "content": "hi" }]
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["reply"], "hello from mock");
    }
}
