pub mod google_calendar;
pub mod handler;
pub mod zoom;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(handler::create_meeting)
        .service(handler::get_meetings)
        .service(handler::update_meeting)
        .service(handler::delete_meeting);
}
