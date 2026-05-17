use crate::prelude::*;

#[derive(Clone, serde::Serialize, FromRow)]
pub struct Account {
    id: i32,
    email: String,
    display_name: Option<String>,
    unread_count: i64,
}
