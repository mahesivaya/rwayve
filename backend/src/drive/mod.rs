pub mod handler;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(handler::upload_file)
        .service(handler::get_files)
        .service(handler::download_file);
}
