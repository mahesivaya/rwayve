use crate::prelude::*;

use crate::security::jwt::create_jwt;
use crate::models::auth::{LoginInput, LoginResponse, RegisterInput};
use crate::models::message::MessageResponse;
use crate::models::user::User;
use bcrypt::{DEFAULT_COST, hash, verify};


#[post("/register")]
pub async fn register(pool: web::Data<PgPool>, data: web::Json<RegisterInput>) -> HttpResponse {
    log_auth("simple message");
    log_auth(format!("User registered: {}", data.email));
    if data.password != data.confirm_password {
        log_auth(format!(
            "Register failed (password mismatch): {}",
            data.email
        ));
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "Passwords do not match" }));
    }
    log_auth(format!("User registered successfully: {}", data.email));

    // 🔥 HASH PASSWORD
    let hashed = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            println!("Hash error: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Password hashing failed" }));
        }
    };

    let result = sqlx::query("INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id")
        .bind(&data.email)
        .bind(&hashed) // ✅ FIXED
        .fetch_one(pool.get_ref())
        .await;

        match result {
            Ok(row) => {
                let user_id: i32 = row.get("id");
        
                // ✅ ALWAYS use same JWT function
                let token = create_jwt(user_id, data.email.clone());
        
                HttpResponse::Ok().json(serde_json::json!({ "token": token }))
            }
        
            Err(e) => {
                println!("DB ERROR: {:?}", e);
        
                if e.to_string().contains("duplicate key") {
                    HttpResponse::BadRequest()
                        .json(serde_json::json!({ "message": "User already exists" }))
                } else {
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "message": "Insert failed" }))
                }
            }
        }
}

#[post("/login")]
async fn login(pool: web::Data<PgPool>, data: web::Json<LoginInput>) -> HttpResponse {
    println!("Login attempt: {}", data.email);

    // ✅ HANDLE DB RESULT PROPERLY
    let user_result =
        sqlx::query_as::<_, User>("SELECT id, email, password FROM users WHERE email = $1")
            .bind(&data.email)
            .fetch_optional(pool.get_ref())
            .await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            println!("User not found");
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
