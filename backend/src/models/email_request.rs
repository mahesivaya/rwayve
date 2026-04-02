use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SendEmailRequest {
    pub account_id: i32,
    pub to: String,
    pub subject: String,
    pub body: String,
}