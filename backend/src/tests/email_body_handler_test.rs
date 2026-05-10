#[cfg(test)]
mod tests {
    use crate::test_support::{insert_local_user, random_email, test_pool};
    use serde_json::json;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn set_env(key: &str, val: &str) {
        unsafe {
            std::env::set_var(key, val);
        }
    }

    fn write_fake_client_secret() -> String {
        let path = std::env::temp_dir().join(format!(
            "rwayve-body-secret-{}.json",
            uuid::Uuid::new_v4()
        ));
        std::fs::write(
            &path,
            br#"{"web":{"client_id":"test","client_secret":"test","redirect_uris":["http://x"]}}"#,
        )
        .unwrap();
        path.to_string_lossy().to_string()
    }

    /// Build a Gmail full-message response with a plaintext body.
    /// extract_body looks at parts[0].body.data (base64 url-safe).
    fn full_message_response(plaintext: &str) -> serde_json::Value {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(plaintext);
        json!({
            "snippet": "snippet fallback",
            "payload": {
                "mimeType": "text/plain",
                "body": { "data": encoded },
                "headers": [],
            }
        })
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn process_account_fills_bodies_for_pending_emails() {
        // Required env: client secret + AES key (encrypt() needs it).
        set_env("GOOGLE_CLIENT_SECRET_PATH", &write_fake_client_secret());
        set_env("AES_KEY", "0123456789abcdef0123456789abcdef");

        let server = MockServer::start().await;
        set_env("GMAIL_API_BASE", &server.uri());
        // Mock Google OAuth refresh — returns an access token immediately.
        set_env("GOOGLE_TOKEN_URL", &format!("{}/token", server.uri()));
        Mock::given(method("POST"))
            .and(path_regex(r"^/token$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "fresh-access",
                "expires_in": 3600,
            })))
            .mount(&server)
            .await;

        // The body fetch endpoint — return same body for any message id.
        Mock::given(method("GET"))
            .and(path_regex(r"^/gmail/v1/users/me/messages/.*"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(full_message_response("hello body")),
            )
            .mount(&server)
            .await;

        // Set up a user + account + a couple of pending emails (empty body).
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "p").await;

        let acc_row = sqlx::query(
            "INSERT INTO email_accounts (email, user_id, access_token, refresh_token, token_expiry, is_active)
             VALUES ($1,$2,$3,$4, NOW() + INTERVAL '1 hour', true)
             RETURNING id",
        )
        .bind(&email)
        .bind(user_id)
        .bind("old")
        .bind("rt-1")
        .fetch_one(&pool)
        .await
        .unwrap();
        let account_id: i32 = sqlx::Row::get(&acc_row, "id");

        for gmail_id in &["msg-1", "msg-2"] {
            sqlx::query(
                "INSERT INTO emails (gmail_id, sender, receiver, subject, body_encrypted, body_iv, account_id)
                 VALUES ($1,$2,$3,$4,'','',$5)",
            )
            .bind(gmail_id)
            .bind("from")
            .bind("to")
            .bind("s")
            .bind(account_id)
            .execute(&pool)
            .await
            .unwrap();
        }

        let n = super::process_account(&pool, account_id, "cid", "csecret", "rt-1")
            .await
            .expect("process_account ok");
        assert_eq!(n, 2);

        // Bodies must now be non-empty + decryptable to "hello body".
        let rows = sqlx::query("SELECT body_iv, body_encrypted FROM emails WHERE account_id = $1")
            .bind(account_id)
            .fetch_all(&pool)
            .await
            .unwrap();
        for r in &rows {
            let iv: String = sqlx::Row::get(r, "body_iv");
            let enc: String = sqlx::Row::get(r, "body_encrypted");
            assert!(!iv.is_empty());
            assert!(!enc.is_empty());
            let plaintext = crate::security::encryption::decrypt(&iv, &enc).unwrap();
            assert_eq!(plaintext, "hello body");
        }

        sqlx::query("DELETE FROM emails WHERE account_id = $1")
            .bind(account_id)
            .execute(&pool)
            .await
            .ok();
        sqlx::query("DELETE FROM email_accounts WHERE id = $1")
            .bind(account_id)
            .execute(&pool)
            .await
            .ok();
        crate::test_support::delete_user(&pool, user_id).await;
    }
}
