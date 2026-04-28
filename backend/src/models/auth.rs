
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // user_id
    pub email: String,
    pub exp: usize,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}
