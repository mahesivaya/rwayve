use crate::prelude::*;

use crate::models::auth::Claims;
use actix_web::HttpRequest;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};


pub fn create_jwt(user_id: i32, email: String) -> String {
    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email,
        exp: expiration,
    };

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
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
