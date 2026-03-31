use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SendEmailRequest {
    pub to: String,
    pub subject: String,
    pub body: String,
}