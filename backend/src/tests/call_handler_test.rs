#[cfg(test)]
mod ws_tests {
    use super::*;
    use crate::test_support::{jwt_for, next_synthetic_user_id};
    use actix_web::App;
    use awc::ws as awsm;
    use futures_util::{SinkExt, StreamExt};
    use std::time::Duration;

    /// Build a `?token=<jwt>` query string for a synthetic user_id. The JWT
    /// signs with the default test secret ("secret"); call_ws will validate
    /// against the same secret because no test sets JWT_SECRET differently.
    fn token_query(user_id: i32) -> String {
        let jwt = jwt_for(user_id, &format!("ws-{user_id}@test.local"));
        format!("token={jwt}")
    }

    /// Wait for the next text frame on `framed`, with a 2s timeout.
    /// Macro form so callers don't have to name the framed type.
    macro_rules! next_text {
        ($framed:expr) => {{
            let timed = tokio::time::timeout(Duration::from_secs(2), $framed.next()).await;
            match timed {
                Ok(Some(Ok(awsm::Frame::Text(bytes)))) => {
                    Some(String::from_utf8(bytes.to_vec()).expect("utf8 text frame"))
                }
                _ => None,
            }
        }};
    }

    #[actix_web::test]
    async fn call_ws_forwards_signal_between_two_clients() {
        let srv = actix_test::start(|| App::new().route("/ws/call", web::get().to(call_ws)));

        let id_a = next_synthetic_user_id();
        let id_b = next_synthetic_user_id();

        let url_a = srv.url(&format!("/ws/call?{}", token_query(id_a)));
        let url_b = srv.url(&format!("/ws/call?{}", token_query(id_b)));
        let (_, mut a) = awc::Client::default().ws(&url_a).connect().await.unwrap();
        let (_, mut b) = awc::Client::default().ws(&url_b).connect().await.unwrap();

        // Give B's `started` hook a tick to register in SESSIONS before A sends.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let signal = serde_json::json!({
            "type": "offer",
            "to": id_b,
            "from": null,
            "sdp": "v=0\r\no=- 1 1 IN IP4 0.0.0.0",
            "candidate": null,
        });
        a.send(awsm::Message::Text(signal.to_string().into()))
            .await
            .unwrap();

        let received = next_text!(b).expect("B got a frame");
        let v: serde_json::Value = serde_json::from_str(&received).unwrap();
        assert_eq!(v["type"], "offer");
        assert_eq!(v["to"], id_b);
        assert_eq!(v["from"], id_a, "server must stamp `from` with sender id");
        assert!(v["sdp"].as_str().unwrap_or("").contains("v=0"));
    }

    #[actix_web::test]
    async fn call_ws_forwards_ice_candidate() {
        let srv = actix_test::start(|| App::new().route("/ws/call", web::get().to(call_ws)));

        let id_a = next_synthetic_user_id();
        let id_b = next_synthetic_user_id();

        let url_a = srv.url(&format!("/ws/call?{}", token_query(id_a)));
        let url_b = srv.url(&format!("/ws/call?{}", token_query(id_b)));
        let (_, mut a) = awc::Client::default().ws(&url_a).connect().await.unwrap();
        let (_, mut b) = awc::Client::default().ws(&url_b).connect().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let signal = serde_json::json!({
            "type": "ice",
            "to": id_b,
            "from": null,
            "sdp": null,
            "candidate": {
                "candidate": "candidate:0 1 UDP 2122252543 192.168.1.1 50000 typ host",
                "sdp_mid": "0",
                "sdp_m_line_index": 0,
                "username_fragment": "abc"
            }
        });
        a.send(awsm::Message::Text(signal.to_string().into()))
            .await
            .unwrap();

        let received = next_text!(b).expect("B got a frame");
        let v: serde_json::Value = serde_json::from_str(&received).unwrap();
        assert_eq!(v["type"], "ice");
        assert_eq!(v["from"], id_a);
        assert_eq!(v["candidate"]["sdp_mid"], "0");
    }

    #[actix_web::test]
    async fn call_ws_silently_drops_signal_when_target_offline() {
        let srv = actix_test::start(|| App::new().route("/ws/call", web::get().to(call_ws)));

        let id_a = next_synthetic_user_id();
        let url_a = srv.url(&format!("/ws/call?{}", token_query(id_a)));
        let (_, mut a) = awc::Client::default().ws(&url_a).connect().await.unwrap();

        let signal = serde_json::json!({
            "type": "offer",
            "to": 999_999_999,
            "from": null,
            "sdp": "v=0",
            "candidate": null,
        });
        a.send(awsm::Message::Text(signal.to_string().into()))
            .await
            .unwrap();

        let res = tokio::time::timeout(Duration::from_millis(200), a.next()).await;
        assert!(
            res.is_err(),
            "sender must not receive an echo when target is offline"
        );
    }

    #[actix_web::test]
    async fn call_ws_ignores_non_json_text_and_keeps_session_open() {
        let srv = actix_test::start(|| App::new().route("/ws/call", web::get().to(call_ws)));

        let id_a = next_synthetic_user_id();
        let id_b = next_synthetic_user_id();

        let url_a = srv.url(&format!("/ws/call?{}", token_query(id_a)));
        let url_b = srv.url(&format!("/ws/call?{}", token_query(id_b)));
        let (_, mut a) = awc::Client::default().ws(&url_a).connect().await.unwrap();
        let (_, mut b) = awc::Client::default().ws(&url_b).connect().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        a.send(awsm::Message::Text("not json at all".into()))
            .await
            .unwrap();

        let signal = serde_json::json!({
            "type": "answer",
            "to": id_b,
            "from": null,
            "sdp": "v=0",
            "candidate": null,
        });
        a.send(awsm::Message::Text(signal.to_string().into()))
            .await
            .unwrap();

        let received = next_text!(b).expect("B got the answer signal");
        let v: serde_json::Value = serde_json::from_str(&received).unwrap();
        assert_eq!(v["type"], "answer");
    }

    // ----- Auth regression tests -----

    #[actix_web::test]
    async fn call_ws_rejects_connection_without_token() {
        let srv = actix_test::start(|| App::new().route("/ws/call", web::get().to(call_ws)));

        let url = srv.url("/ws/call");
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
    async fn call_ws_rejects_invalid_token() {
        let srv = actix_test::start(|| App::new().route("/ws/call", web::get().to(call_ws)));

        let url = srv.url("/ws/call?token=not-a-real-jwt");
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

    /// SECURITY REGRESSION: ensures user_id comes from the JWT, never from a
    /// query string. Without this, an attacker could pass `?user_id=<victim>`
    /// and impersonate another user. Test mounts attacker as id_a (per JWT)
    /// while passing `&user_id=<victim>` in the URL. The signal should be
    /// stamped with id_a (the JWT subject), not the victim id.
    #[actix_web::test]
    #[serial_test::serial]
    async fn call_ws_user_id_comes_from_jwt_not_query_param() {
        let srv = actix_test::start(|| App::new().route("/ws/call", web::get().to(call_ws)));

        let id_a = next_synthetic_user_id();
        let id_b = next_synthetic_user_id();
        let victim = next_synthetic_user_id();

        // Attacker A connects with their own JWT but tries to spoof
        // user_id=victim via the legacy query parameter.
        let url_a = srv.url(&format!(
            "/ws/call?{}&user_id={}",
            token_query(id_a),
            victim
        ));
        let url_b = srv.url(&format!("/ws/call?{}", token_query(id_b)));
        let (_, mut a) = awc::Client::default().ws(&url_a).connect().await.unwrap();
        let (_, mut b) = awc::Client::default().ws(&url_b).connect().await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        a.send(awsm::Message::Text(
            serde_json::json!({
                "type": "offer",
                "to": id_b,
                "from": null,
                "sdp": "v=0",
                "candidate": null,
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();

        let received = next_text!(b).expect("B got the signal");
        let v: serde_json::Value = serde_json::from_str(&received).unwrap();
        assert_eq!(
            v["from"], id_a,
            "from must come from JWT claims, NEVER from spoofed user_id query param"
        );
        assert_ne!(
            v["from"], victim,
            "spoofed user_id query param must be ignored"
        );
    }
}
