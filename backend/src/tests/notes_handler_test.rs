#[cfg(test)]
mod tests {
    use crate::notes::handler::{create_note, delete_note, list_notes, update_note};
    use crate::test_support::{delete_user, insert_local_user, jwt_for, random_email, test_pool};
    use actix_web::{App, http::StatusCode, test, web};
    use serde_json::json;

    #[actix_web::test]
    async fn list_notes_requires_auth() {
        let pool = test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(list_notes),
        )
        .await;
        let req = test::TestRequest::get().uri("/notes").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn notes_full_crud_round_trip() {
        let pool = test_pool().await;
        let email = random_email();
        let user_id = insert_local_user(&pool, &email, "p").await;
        let auth = format!("Bearer {}", jwt_for(user_id, &email));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(list_notes)
                .service(create_note)
                .service(update_note)
                .service(delete_note),
        )
        .await;

        // create
        let req = test::TestRequest::post()
            .uri("/notes")
            .insert_header(("Authorization", auth.clone()))
            .set_json(json!({ "title": "t1", "content": "c1" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let created: serde_json::Value = test::read_body_json(resp).await;
        let note_id = created["id"].as_i64().expect("id");

        // list
        let req = test::TestRequest::get()
            .uri("/notes")
            .insert_header(("Authorization", auth.clone()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let listed: serde_json::Value = test::read_body_json(resp).await;
        assert!(
            listed
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["id"] == note_id)
        );

        // update
        let req = test::TestRequest::put()
            .uri(&format!("/notes/{}", note_id))
            .insert_header(("Authorization", auth.clone()))
            .set_json(json!({ "title": "t2", "content": "c2" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // delete
        let req = test::TestRequest::delete()
            .uri(&format!("/notes/{}", note_id))
            .insert_header(("Authorization", auth.clone()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // delete again should 404
        let req = test::TestRequest::delete()
            .uri(&format!("/notes/{}", note_id))
            .insert_header(("Authorization", auth))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        delete_user(&pool, user_id).await;
    }
}
