use crate::prelude::*;

#[derive(serde::Serialize, FromRow)]
pub struct Account {
    id: i32,
    email: String,
}
