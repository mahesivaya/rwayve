pub use actix_web::HttpRequest;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
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

// 🔥 Decode JWT
pub fn decode_jwt(token: &str) -> Option<Claims> {
    let secret = std::env::var("JWT_SECRET").unwrap_or("secret".into());

    let result = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    );

    match result {
        Ok(data) => Some(data.claims),
        Err(e) => {
            println!("❌ JWT decode error: {}", e);
            None
        }
    }
}

pub fn get_user_id_from_request(req: &HttpRequest) -> Option<i32> {
    let header = req.headers().get("Authorization")?.to_str().ok()?;

    // Expect: "Bearer <token>"
    let token = header.strip_prefix("Bearer ")?;

    let claims = decode_jwt(token)?;

    Some(claims.sub)
}
