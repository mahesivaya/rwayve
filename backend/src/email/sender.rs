use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::env;
use tracing::{error, info};

pub(crate) fn clean_mailbox(value: &str) -> String {
    value
        .split_once('#')
        .map_or(value, |(before_comment, _)| before_comment)
        .trim()
        .to_string()
}

pub async fn send_mail(to: &str, subject: &str, body: &str) -> Result<(), String> {
    let host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST missing".to_string())?;
    let port: u16 = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .map_err(|_| "SMTP_PORT invalid".to_string())?;
    let user = env::var("SMTP_USER").map_err(|_| "SMTP_USER missing".to_string())?;
    let pass = env::var("SMTP_PASS").map_err(|_| "SMTP_PASS missing".to_string())?;
    let from = env::var("SMTP_FROM").unwrap_or_else(|_| user.clone());

    let from_parsed = clean_mailbox(&from)
        .parse()
        .map_err(|e| format!("SMTP_FROM invalid: {e:?}"))?;
    let to_parsed = clean_mailbox(to)
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
