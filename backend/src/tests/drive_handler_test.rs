#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_pool;
    use actix_web::{App, http::StatusCode, test, web};

    #[actix_web::test]
    async fn upload_requires_auth() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .service(upload_file),
        )
        .await;
        let req = test::TestRequest::post().uri("/files/upload").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
