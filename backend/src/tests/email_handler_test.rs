#[cfg(test)]
mod tests {
    use crate::routes::email::get_emails;
    use crate::test_support::{insert_local_user, jwt_for, random_email, test_pool};
    use actix_web::{App, http::StatusCode, test, web};
    use serde_json::Value;
    use sqlx::Row;

    #[actix_web::test]
    #[serial_test::serial]
    async fn test_get_emails_pagination_flow() {
        let pool = test_pool().await;
        let email_addr = random_email();
        let user_id = insert_local_user(&pool, &email_addr, "password").await;
        let jwt = jwt_for(user_id, &email_addr);

        // 1. Setup: Create a linked email account
        let account_id: i32 = sqlx::query(
            "INSERT INTO email_accounts (email, user_id, provider) VALUES ($1, $2, 'google') RETURNING id"
        )
        .bind(&email_addr)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .get("id");

        // 2. Setup: Insert 52 emails with descending timestamps
        let base_time = chrono::Utc::now().naive_utc();
        for i in 0..52 {
            sqlx::query(
                "INSERT INTO emails (gmail_id, account_id, subject, sender, receiver, created_at, body_encrypted, body_iv)
                 VALUES ($1, $2, $3, $4, $5, $6, '', '')"
            )
            .bind(format!("msg_{}", i))
            .bind(account_id)
            .bind(format!("Email #{}", i))
            .bind("sender@rwayve.test")
            .bind(&email_addr) // receiver matches account email for 'inbox' filter
            .bind(base_time - chrono::Duration::minutes(i))
            .execute(&pool)
            .await
            .unwrap();
        }

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(web::Data::new(None::<crate::cache::Cache>))
                .service(get_emails),
        )
        .await;

        // --- STEP 1: FETCH PAGE 1 ---
        let req = test::TestRequest::get()
            .uri("/emails?folder=inbox")
            .insert_header(("Authorization", format!("Bearer {}", jwt)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify 'x-has-more' header
        let has_more = resp.headers().get("x-has-more").unwrap().to_str().unwrap();
        assert_eq!(has_more, "true");

        let body: Vec<Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 50, "First page should respect 50 item limit");
        assert_eq!(body[0]["subject"], "Email #0");

        // --- STEP 2: FETCH PAGE 2 USING KEYSET ---
        // Get markers from the 50th email (index 49)
        let last_email = &body[49];
        let last_id = last_email["id"].as_i64().unwrap();
        let last_created_str = last_email["created_at"].as_str().unwrap();

        // Parse the RFC3339 timestamp returned by the API
        let last_created_ts = chrono::DateTime::parse_from_rfc3339(last_created_str)
            .unwrap()
            .timestamp();

        let uri = format!(
            "/emails?folder=inbox&before={}&before_id={}",
            last_created_ts, last_id
        );

        let req_p2 = test::TestRequest::get()
            .uri(&uri)
            .insert_header(("Authorization", format!("Bearer {}", jwt)))
            .to_request();

        let resp_p2 = test::call_service(&app, req_p2).await;
        assert_eq!(resp_p2.status(), StatusCode::OK);

        // Verify 'x-has-more' is now false
        let has_more_2 = resp_p2
            .headers()
            .get("x-has-more")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(has_more_2, "false");

        let body_2: Vec<Value> = test::read_body_json(resp_p2).await;
        assert_eq!(
            body_2.len(),
            2,
            "Second page should contain the remaining 2 items"
        );
        assert_eq!(body_2[0]["subject"], "Email #50");
        assert_eq!(body_2[1]["subject"], "Email #51");
    }
}
