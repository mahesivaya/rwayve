use crate::prelude::*;

#[derive(Serialize, sqlx::FromRow)]
struct Email {
    id: i32,
    subject: Option<String>,
    sender: Option<String>,
    receiver: Option<String>,
    body_encrypted: String,
    body_iv: String,
    account_id: Option<i32>,
    created_at: Option<NaiveDateTime>,
}
