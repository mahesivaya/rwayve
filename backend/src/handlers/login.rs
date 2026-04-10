use crate::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration as ChronoDuration};
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Deserialize)]
struct LoginInput {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: i32,
    email: String,
    exp: usize,
}

#[derive(FromRow)]
struct User {
    id: i32,
    email: String,
    password: String,
}

#[post("/api/login")]
async fn login(
    pool: web::Data<PgPool>,
    data: web::Json<LoginInput>,
) -> HttpResponse {

    println!("Login attempt: {}", data.email);
    log_auth(format!("Login attempt username {}", data.email));
    // ✅ HANDLE DB RESULT PROPERLY
    let user_result = sqlx::query_as::<_, User>(
        "SELECT id, email, password FROM users WHERE email = $1"
    )
    .bind(&data.email)
    .fetch_optional(pool.get_ref())
    .await;

    let user = match user_result {
        Ok(Some(user)) => {
        log_auth(format!("User login: {}", data.email));
        user}
        Ok(None) => {
            println!("User not found");
            log_auth(format!("User login not found: {}", data.email));
            return HttpResponse::Unauthorized().json(MessageResponse {
                message: "Invalid credentials".to_string(),
            });
        }
        Err(e) => {
            println!("DB ERROR: {:?}", e);
            return HttpResponse::InternalServerError().json(MessageResponse {
                message: "Database error".to_string(),
            });
        }
    };

    // ✅ SAFE bcrypt check
    let valid = match verify(&data.password, &user.password) {
        Ok(v) => v,
        Err(e) => {
            println!("bcrypt verify error: {:?}", e);

            // 🔥 THIS IS YOUR CURRENT 500 ROOT CAUSE
            return HttpResponse::InternalServerError().json(MessageResponse {
                message: "Password verification failed".to_string(),
            });
        }
    };

    if !valid {
        return HttpResponse::Unauthorized().json(MessageResponse {
            message: "Invalid credentials".to_string(),
        });
    }

    // ✅ CREATE TOKEN
    let token = create_jwt(user.id, user.email.clone());

    HttpResponse::Ok().json(LoginResponse { token })
}



fn create_jwt(user_id: i32, email: String) -> String {
    let expiration = Utc::now()
        .checked_add_signed(ChronoDuration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    )
    .unwrap()
}

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Email Import Running")
}