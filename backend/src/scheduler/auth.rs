use actix_web::{HttpRequest, HttpResponse};

pub fn get_user_id(req: &HttpRequest) -> Result<i32, HttpResponse> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| HttpResponse::Unauthorized().body("Missing token"))?;

    let decoded = crate::security::jwt::decode_jwt(token)
        .ok_or_else(|| HttpResponse::Unauthorized().body("Invalid token"))?;

    Ok(decoded.sub)
}
