use serde_json::json;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
mod collector;
use crate::collector::{LogCollector, submit_log};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn collect_log(
    collector: web::Data<LogCollector>,
    log: web::Json<String>,
) -> impl Responder {
    match submit_log(&collector, &log.into_inner()) {
        Ok(id) => HttpResponse::Ok().json(json!({ "status": "ok", "id": id })),
        Err(_) => HttpResponse::BadRequest().json(json!({ "status": "error", "message": "Invalid log format" })),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let collector = web::Data::new(LogCollector::new());

    HttpServer::new(move || {
        App::new()
        .app_data(collector.clone())
        .service(
            web::scope("/backend")
                .route("/", web::get().to(index))
                .route("/import_log", web::post().to(collect_log))
        )
    })
    .bind(("127.0.0.1", 4200)).expect("Failed to bind to address").run().await
}