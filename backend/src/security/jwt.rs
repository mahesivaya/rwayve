use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::warn;

fn default_account_type() -> String {
    "personal".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub email: String,
    #[serde(default = "default_account_type")]
    pub account_type: String,
    pub exp: usize,
}

// 🔥 CREATE JWT
pub fn create_jwt(user_id: i32, email: String) -> String {
    create_jwt_for_account(user_id, email, "personal".to_string())
}

pub fn create_jwt_for_account(user_id: i32, email: String, account_type: String) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or("secret".into());

    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email,
        account_type,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_else(|e| panic!("JWT encode failed: {e}"))
}

// 🔥 DECODE JWT
pub fn decode_jwt(token: &str) -> Option<Claims> {
    let secret = std::env::var("JWT_SECRET").unwrap_or("secret".into());

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => Some(data.claims),
        Err(e) => {
            warn!(target: "auth", error = %e, "jwt decode failed");
            None
        }
    }
}

// 🔥 Extract user from request
use actix_web::HttpRequest;

pub fn get_user_id_from_request(req: &HttpRequest) -> Option<i32> {
    let header = req.headers().get("Authorization")?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;
    let claims = decode_jwt(token)?;
    Some(claims.sub)
}
