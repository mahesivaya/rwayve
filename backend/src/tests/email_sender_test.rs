/// Plaintext-SMTP variant used by tests against MailHog.
/// In production we always go through `send_mail` which negotiates STARTTLS.
#[cfg(test)]
pub(crate) async fn send_mail_plaintext_for_tests(
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    let host = std::env::var("SMTP_HOST").map_err(|_| "SMTP_HOST missing".to_string())?;
    let port: u16 = std::env::var("SMTP_PORT")
        .unwrap_or_else(|_| "1025".to_string())
        .parse()
        .map_err(|_| "SMTP_PORT invalid".to_string())?;
    let from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@test.local".to_string());

    let from_parsed = from
        .parse()
        .map_err(|e| format!("SMTP_FROM invalid: {e:?}"))?;
    let to_parsed = to.parse().map_err(|e| format!("recipient invalid: {e:?}"))?;

    let email = lettre::Message::builder()
        .from(from_parsed)
        .to(to_parsed)
        .subject(subject)
        .header(lettre::message::header::ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| format!("message build failed: {e:?}"))?;

    let mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor> =
        lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(&host)
            .port(port)
            .build();

    lettre::AsyncTransport::send(&mailer, email)
        .await
        .map(|_| ())
        .map_err(|e| format!("send failed: {e:?}"))
}
