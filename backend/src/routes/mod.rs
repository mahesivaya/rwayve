pub mod account;
pub mod auth;
pub mod email;
pub mod health;
pub mod user;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health)
        .service(health::ready)
        .service(auth::register)
        .service(auth::login)
        .service(auth::logout)
        .service(auth::forgot_password)
        .service(auth::reset_password)
        .service(user::change_password)
        .service(user::admin_list_organizations)
        .service(user::admin_create_organization)
        .service(user::admin_create_user)
        .service(account::get_accounts)
        .service(user::get_user_by_email)
        .service(user::get_all_users)
        .service(user::get_profile)
        .service(user::update_profile)
        .service(account::delete_account);
}
