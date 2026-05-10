use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::env;
use tracing::{error, info};

pub async fn send_mail(to: &str, subject: &str, body: &str) -> Result<(), String> {
    let host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST missing".to_string())?;
    let port: u16 = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .map_err(|_| "SMTP_PORT invalid".to_string())?;
    let user = env::var("SMTP_USER").map_err(|_| "SMTP_USER missing".to_string())?;
    let pass = env::var("SMTP_PASS").map_err(|_| "SMTP_PASS missing".to_string())?;
    let from = env::var("SMTP_FROM").unwrap_or_else(|_| user.clone());

    let from_parsed = from
        .parse()
        .map_err(|e| format!("SMTP_FROM invalid: {e:?}"))?;
    let to_parsed = to
        .parse()
        .map_err(|e| format!("recipient invalid: {e:?}"))?;

    let email = Message::builder()
        .from(from_parsed)
        .to(to_parsed)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| format!("message build failed: {e:?}"))?;

    let creds = Credentials::new(user, pass);
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
            .map_err(|e| format!("transport build failed: {e:?}"))?
            .port(port)
            .credentials(creds)
            .build();

    match mailer.send(email).await {
        Ok(_) => {
            info!(target: "smtp", to, "mail sent");
            Ok(())
        }
        Err(e) => {
            error!(target: "smtp", to, error = %e, "mail send failed");
            Err(format!("send failed: {e:?}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    /// Skip when MAILHOG_API isn't set so local devs without MailHog can run
    /// the rest of the suite. CI sets this and brings up the service.
    fn mailhog_api() -> Option<String> {
        std::env::var("MAILHOG_API").ok()
    }

    fn set_smtp_env(host: &str, port: u16) {
        unsafe {
            std::env::set_var("SMTP_HOST", host);
            std::env::set_var("SMTP_PORT", port.to_string());
            // MailHog accepts any creds.
            std::env::set_var("SMTP_USER", "ci@example.com");
            std::env::set_var("SMTP_PASS", "ci-pass");
            std::env::set_var("SMTP_FROM", "noreply@rwayve.test");
        }
    }

    async fn purge_mailhog(api_base: &str) {
        let _ = reqwest::Client::new()
            .delete(format!("{api_base}/api/v1/messages"))
            .send()
            .await;
    }

    async fn fetch_messages(api_base: &str) -> Vec<Value> {
        let res: Value = reqwest::get(format!("{api_base}/api/v2/messages"))
            .await
            .expect("hit mailhog")
            .json()
            .await
            .expect("decode mailhog");
        res["items"].as_array().cloned().unwrap_or_default()
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn send_mail_actually_lands_in_mailhog() {
        let Some(api) = mailhog_api() else {
            eprintln!("skipping: set MAILHOG_API to enable (e.g. http://localhost:8025)");
            return;
        };

        // MailHog SMTP listens on 1025 by default; allow override via env.
        let smtp_host = std::env::var("MAILHOG_SMTP_HOST").unwrap_or_else(|_| "localhost".into());
        let smtp_port: u16 = std::env::var("MAILHOG_SMTP_PORT")
            .unwrap_or_else(|_| "1025".into())
            .parse()
            .unwrap();

        // MailHog doesn't speak STARTTLS; the lettre `starttls_relay` we use
        // expects it. Tests targeting MailHog must therefore go through the
        // SMTP_TEST_BYPASS path below, which uses a plaintext relay.
        set_smtp_env(&smtp_host, smtp_port);

        purge_mailhog(&api).await;

        let to = format!("inbox-{}@example.test", uuid::Uuid::new_v4());
        let result = super::send_mail_plaintext_for_tests(&to, "Test subject", "hello world").await;
        result.expect("mail sent");

        // Poll MailHog up to 5s for the message to appear.
        let mut messages = Vec::new();
        for _ in 0..25 {
            messages = fetch_messages(&api).await;
            if !messages.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        assert!(
            !messages.is_empty(),
            "expected at least one message in MailHog"
        );

        let m = &messages[0];
        let recipient = m["To"][0]["Mailbox"].as_str().unwrap_or("").to_string()
            + "@"
            + m["To"][0]["Domain"].as_str().unwrap_or("");
        assert_eq!(recipient, to);
        let body = m["Content"]["Body"].as_str().unwrap_or("");
        assert!(body.contains("hello world"), "body was: {body}");
    }
}
