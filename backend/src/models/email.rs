use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Email {
    pub id: Option<i32>,
    pub gmail_id: String,
    pub sender: String,
    pub receiver: String,
    pub subject: String,
    pub body: Option<String>,
    pub account_id: i32,
    pub created_at: NaiveDateTime,
}