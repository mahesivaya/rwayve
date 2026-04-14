use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,     // user_id
    pub email: String,
    pub exp: usize,
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