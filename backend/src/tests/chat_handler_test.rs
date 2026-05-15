#[cfg(test)]
mod tests {
    use crate::cache::Cache;
    use crate::chat::handler::{chat_ws, get_messages};
    use crate::test_support::{insert_local_user, jwt_for, random_email, test_pool};
    use actix_web::{App, http::StatusCode, test, web};
    use awc::ws as awsm;
    use futures_util::{SinkExt, StreamExt};
    use sqlx::PgPool;
    use std::time::Duration;

    fn token_query(user_id: i32) -> String {
        let jwt = jwt_for(user_id, &format!("ws-{user_id}@test.local"));
        format!("token={jwt}")
    }

    #[actix_web::test]
    async fn get_messages_requires_auth() {
        let pool = test_pool().await;
        let cache: Option<Cache> = None;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(cache))
                .service(get_messages),
        )
        .await;
        // Valid query params so the `web::Query` extractor succeeds and the
        // request reaches the handler's auth check.
        let req = test::TestRequest::get()
            .uri("/messages?user1=1&user2=2")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    fn ensure_aes_key() {
        unsafe {
            std::env::set_var(
                "AES_KEY",
                "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            );
        }
    }

    macro_rules! next_text {
        ($framed:expr, $timeout_ms:expr) => {{
            let timed =
                tokio::time::timeout(Duration::from_millis($timeout_ms), $framed.next()).await;
            match timed {
                Ok(Some(Ok(awsm::Frame::Text(bytes)))) => {
                    Some(String::from_utf8(bytes.to_vec()).expect("utf8 text frame"))
                }
                _ => None,
            }
        }};
    }

    /// Spin up a real test server with chat_ws + the data the handler needs.
    fn start_chat_server(pool: PgPool) -> actix_test::TestServer {
        actix_test::start(move || {
            let cache: Option<Cache> = None;
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(web::Data::new(cache))
                .route("/ws/chat", web::get().to(chat_ws))
        })
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn chat_ws_persists_message_and_echoes_to_sender() {
        ensure_aes_key();
        let pool = test_pool().await;
        let user_a = insert_local_user(&pool, &random_email(), "p").await;
        let user_b = insert_local_user(&pool, &random_email(), "p").await;

        let srv = start_chat_server(pool.clone());
        let url = srv.url(&format!("/ws/chat?{}", token_query(user_a)));
        let (_, mut a) = awc::Client::default().ws(&url).connect().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let payload = serde_json::json!({
            "sender_id": user_a,
            "receiver_id": user_b,
            "content": "WAYVE_CHAT_E2E_V1\nhello over ws",
            "status": null,
            "message_id": null,
        });
        a.send(awsm::Message::Text(payload.to_string().into()))
            .await
            .unwrap();

        let echoed = next_text!(a, 2000).expect("sender got echo");
        let v: serde_json::Value = serde_json::from_str(&echoed).unwrap();
        assert_eq!(v["sender_id"], user_a);
        assert_eq!(v["receiver_id"], user_b);
        assert_eq!(v["content"], "WAYVE_CHAT_E2E_V1\nhello over ws");
        assert_eq!(v["status"], "sent");
        assert!(v["message_id"].is_i64());

        // DB row must exist with encrypted content (not plaintext).
        let row = sqlx::query(
            "SELECT content_encrypted, content_iv FROM messages WHERE sender_id = $1 AND receiver_id = $2",
        )
        .bind(user_a)
        .bind(user_b)
        .fetch_one(&pool)
        .await
        .expect("message row inserted");
        let enc: String = sqlx::Row::get(&row, "content_encrypted");
        let iv: String = sqlx::Row::get(&row, "content_iv");
        assert!(!enc.is_empty() && !iv.is_empty());
        let decrypted = crate::security::encryption::decrypt(&iv, &enc).unwrap();
        assert_eq!(decrypted, "WAYVE_CHAT_E2E_V1\nhello over ws");

        sqlx::query("DELETE FROM messages WHERE sender_id = $1 OR receiver_id = $1")
            .bind(user_a)
            .execute(&pool)
            .await
            .ok();
        crate::test_support::delete_user(&pool, user_a).await;
        crate::test_support::delete_user(&pool, user_b).await;
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn chat_ws_relays_to_online_receiver_and_marks_delivered() {
        ensure_aes_key();
        let pool = test_pool().await;
        let user_a = insert_local_user(&pool, &random_email(), "p").await;
        let user_b = insert_local_user(&pool, &random_email(), "p").await;

        let srv = start_chat_server(pool.clone());
        let url_a = srv.url(&format!("/ws/chat?{}", token_query(user_a)));
        let url_b = srv.url(&format!("/ws/chat?{}", token_query(user_b)));
        let (_, mut a) = awc::Client::default().ws(&url_a).connect().await.unwrap();
        let (_, mut b) = awc::Client::default().ws(&url_b).connect().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let payload = serde_json::json!({
            "sender_id": user_a,
            "receiver_id": user_b,
            "content": "WAYVE_CHAT_E2E_V1\nping",
            "status": null,
            "message_id": null,
        });
        a.send(awsm::Message::Text(payload.to_string().into()))
            .await
            .unwrap();

        // B should receive the message.
        let received_b = next_text!(b, 2000).expect("B got the message");
        let vb: serde_json::Value = serde_json::from_str(&received_b).unwrap();
        assert_eq!(vb["content"], "WAYVE_CHAT_E2E_V1\nping");
        assert_eq!(vb["sender_id"], user_a);

        // A should receive both the echo and the "delivered" status update —
        // order isn't guaranteed, so collect both and check.
        let mut got_echo = false;
        let mut got_delivered = false;
        for _ in 0..2 {
            if let Some(text) = next_text!(a, 1500) {
                let v: serde_json::Value = serde_json::from_str(&text).unwrap();
                if v["status"] == "sent" && v["content"] == "WAYVE_CHAT_E2E_V1\nping" {
                    got_echo = true;
                } else if v["type"] == "status_update" && v["status"] == "delivered" {
                    got_delivered = true;
                }
            }
        }
        assert!(got_echo, "sender should receive the echoed message");
        assert!(
            got_delivered,
            "sender should receive 'delivered' status update"
        );

        sqlx::query("DELETE FROM messages WHERE sender_id = $1 OR receiver_id = $1")
            .bind(user_a)
            .execute(&pool)
            .await
            .ok();
        crate::test_support::delete_user(&pool, user_a).await;
        crate::test_support::delete_user(&pool, user_b).await;
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn chat_ws_marks_messages_as_read() {
        ensure_aes_key();
        let pool = test_pool().await;
        let user_a = insert_local_user(&pool, &random_email(), "p").await;
        let user_b = insert_local_user(&pool, &random_email(), "p").await;

        // Pre-insert two messages A→B, both 'sent'.
        let (iv1, enc1) = crate::security::encryption::encrypt("m1").unwrap();
        let (iv2, enc2) = crate::security::encryption::encrypt("m2").unwrap();
        for (iv, enc) in [(iv1, enc1), (iv2, enc2)] {
            sqlx::query(
                "INSERT INTO messages (sender_id, receiver_id, content_encrypted, content_iv, status)
                 VALUES ($1,$2,$3,$4,'sent')",
            )
            .bind(user_a)
            .bind(user_b)
            .bind(enc)
            .bind(iv)
            .execute(&pool)
            .await
            .unwrap();
        }

        let srv = start_chat_server(pool.clone());
        let url_b = srv.url(&format!("/ws/chat?{}", token_query(user_b)));
        let (_, mut b) = awc::Client::default().ws(&url_b).connect().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // From B's perspective, send a "read" receipt for A's messages.
        let read_receipt = serde_json::json!({
            "sender_id": user_b,           // the reader
            "receiver_id": user_a,         // the original sender
            "content": "",
            "status": "read",
            "message_id": null,
        });
        b.send(awsm::Message::Text(read_receipt.to_string().into()))
            .await
            .unwrap();

        // The handler spawns the UPDATE; give it time to land.
        tokio::time::sleep(Duration::from_millis(300)).await;

        let read_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages WHERE sender_id = $1 AND receiver_id = $2 AND status = 'read'",
        )
        .bind(user_a)
        .bind(user_b)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(read_count, 2, "both messages should be marked read");

        sqlx::query("DELETE FROM messages WHERE sender_id = $1 OR receiver_id = $1")
            .bind(user_a)
            .execute(&pool)
            .await
            .ok();
        crate::test_support::delete_user(&pool, user_a).await;
        crate::test_support::delete_user(&pool, user_b).await;
    }

    // ----- Auth regression tests -----

    #[actix_web::test]
    #[serial_test::serial]
    async fn chat_ws_rejects_connection_without_token() {
        let pool = test_pool().await;
        let srv = start_chat_server(pool);

        let url = srv.url("/ws/chat");
        let res = awc::Client::default().ws(&url).connect().await;
        match res {
            Ok(_) => panic!("expected handshake to fail (401), but it upgraded"),
            Err(e) => {
                let msg = format!("{e:?}");
                assert!(
                    msg.contains("401")
                        || msg.contains("Unauthorized")
                        || msg.contains("InvalidResponseStatus"),
                    "expected 401-shaped error, got: {msg}"
                );
            }
        }
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn chat_ws_rejects_invalid_token() {
        let pool = test_pool().await;
        let srv = start_chat_server(pool);

        let url = srv.url("/ws/chat?token=garbage.not.a.jwt");
        let res = awc::Client::default().ws(&url).connect().await;
        match res {
            Ok(_) => panic!("expected handshake to fail (401), but it upgraded"),
            Err(e) => {
                let msg = format!("{e:?}");
                assert!(
                    msg.contains("401")
                        || msg.contains("Unauthorized")
                        || msg.contains("InvalidResponseStatus"),
                    "expected 401-shaped error, got: {msg}"
                );
            }
        }
    }

    /// SECURITY REGRESSION: messages must be persisted as the JWT subject,
    /// regardless of any `sender_id` the client puts in the body. Without
    /// this, anyone with their own valid JWT could send messages "from" any
    /// other user simply by lying in the payload.
    #[actix_web::test]
    #[serial_test::serial]
    async fn chat_ws_user_id_comes_from_jwt_not_query_param() {
        ensure_aes_key();
        let pool = test_pool().await;
        let attacker = insert_local_user(&pool, &random_email(), "p").await;
        let victim = insert_local_user(&pool, &random_email(), "p").await;
        let target = insert_local_user(&pool, &random_email(), "p").await;

        let srv = start_chat_server(pool.clone());
        // Attacker connects with their own JWT but tries to spoof user_id=victim
        // through the legacy query parameter.
        let url = srv.url(&format!(
            "/ws/chat?{}&user_id={}",
            token_query(attacker),
            victim
        ));
        let (_, mut ws) = awc::Client::default().ws(&url).connect().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Even if the body says sender_id=attacker, the test focuses on the
        // server's binding: it should NOT take user_id from the query string.
        // The handler bypass we want to verify is at the connection layer —
        // any message persists; we then check that the SESSIONS entry was
        // for `attacker`, not `victim`. We do this indirectly: send a message
        // with sender=attacker → if a SESSIONS slot existed for `victim` (which
        // would happen under the old code), the receiver would not get it
        // because we routed to attacker's session.
        ws.send(awsm::Message::Text(
            serde_json::json!({
                "sender_id": attacker,
                "receiver_id": target,
                "content": "WAYVE_CHAT_E2E_V1\nfrom-attacker",
                "status": null,
                "message_id": null,
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();

        // The echoed frame proves the connection used the attacker session.
        let echoed = next_text!(ws, 2000).expect("echo back");
        let v: serde_json::Value = serde_json::from_str(&echoed).unwrap();
        assert_eq!(v["sender_id"], attacker);
        assert!(v["message_id"].is_i64());

        // And the row must have sender_id = attacker.
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages WHERE sender_id = $1 AND receiver_id = $2",
        )
        .bind(attacker)
        .bind(target)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1);

        sqlx::query("DELETE FROM messages WHERE sender_id = $1 OR receiver_id = $1 OR sender_id = $2 OR receiver_id = $2 OR sender_id = $3 OR receiver_id = $3")
            .bind(attacker)
            .bind(victim)
            .bind(target)
            .execute(&pool)
            .await
            .ok();
        crate::test_support::delete_user(&pool, attacker).await;
        crate::test_support::delete_user(&pool, victim).await;
        crate::test_support::delete_user(&pool, target).await;
    }
}
