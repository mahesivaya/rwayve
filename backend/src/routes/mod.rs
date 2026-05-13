pub mod account;
pub mod auth;
pub mod email;
pub mod user;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(auth::register)
        .service(auth::login)
        .service(auth::forgot_password)
        .service(auth::reset_password)
        .service(user::change_password)
        .service(account::get_accounts)
        .service(user::get_user_by_email)
        .service(user::get_all_users)
        .service(user::get_profile)
        .service(user::update_profile)
        .service(account::delete_account);
}
