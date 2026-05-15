#[cfg(test)]
mod tests {
    use crate::routes::account::{delete_account, get_accounts};
    use crate::test_support::test_pool;
    use actix_web::{App, http::StatusCode, test, web};

    #[actix_web::test]
    async fn get_accounts_requires_auth() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .service(get_accounts),
        )
        .await;
        let req = test::TestRequest::get().uri("/accounts").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn delete_account_requires_auth() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .service(delete_account),
        )
        .await;
        let req = test::TestRequest::delete().uri("/accounts/1").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
