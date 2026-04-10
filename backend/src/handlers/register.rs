use crate::prelude::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Utc, Duration as ChronoDuration};
use jsonwebtoken::{encode, EncodingKey, Header};



#[derive(FromRow)]
struct User {
    id: i32,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct RegisterInput {
    email: String,
    password: String,
    confirm_password: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: i32,
    email: String,
    exp: usize,
}


#[post("/api/register")]
async fn register(
    pool: web::Data<PgPool>,
    data: web::Json<RegisterInput>,
) -> HttpResponse {
    log_auth("simple message");
    log_auth(format!("User registered: {}", data.email));
    if data.password != data.confirm_password {
        log_auth(format!("Register failed (password mismatch): {}", data.email));
        return HttpResponse::BadRequest().json(
            serde_json::json!({ "message": "Passwords do not match" })
        );
    }
    log_auth(&format!("User registered successfully: {}", data.email));

    // 🔥 HASH PASSWORD
    let hashed = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            println!("Hash error: {:?}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({ "message": "Password hashing failed" })
            );
        }
    };

    let result = sqlx::query(
        "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id"
    )
    .bind(&data.email)
    .bind(&hashed) // ✅ FIXED
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(row) => {
            let user_id: i32 = row.get("id");

            let claims = Claims {
                sub: user_id,
                email: data.email.clone(),
                exp: (Utc::now() + ChronoDuration::hours(24)).timestamp() as usize,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret("secret".as_ref()),
            ).unwrap();

            HttpResponse::Ok().json(
                serde_json::json!({ "token": token })
            )
        }

        Err(e) => {
            println!("DB ERROR: {:?}", e);

            if e.to_string().contains("duplicate key") {
                HttpResponse::BadRequest().json(
                    serde_json::json!({ "message": "User already exists" })
                )
            } else {
                HttpResponse::InternalServerError().json(
                    serde_json::json!({ "message": "Insert failed" })
                )
            }
        }
    }
}