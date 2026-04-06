use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i32,
    pub email: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub last_sync: Option<i64>,
    pub is_active: Option<bool>,
}