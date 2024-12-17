mod global;
mod database;
mod storage;
mod collector;
mod batch_maker;
mod message_queue;
mod rules;

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use crate::collector::{LogCollector, ParseLogError, process_logs};
use crate::batch_maker::{create_batch};
use serde_json::json;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn import_log(collector: web::Data<LogCollector>, log: web::Json<String>, account_id: String) -> impl Responder {
    match create_batch(&log.into_inner()).await {
        Ok(_) => {
            match process_logs(&collector, account_id).await {
                Ok(_) => HttpResponse::Ok().json(json!({ "status": "ok" })),
                Err(e) => {
                    eprintln!("Error processing logs: {}", e);
                    match e {
                        ParseLogError::DatabaseError(msg) => {
                            HttpResponse::InternalServerError().json(json!({
                                "status": "error",
                                "message": "Database error",
                                "details": msg
                            }))
                        },
                        _ => HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Failed to process logs"
                        }))
                    }
                }
            }
        },
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
                    .route("/import_log", web::post().to(import_log))
            )
    })
    .bind(("127.0.0.1", 4200))?
    .run()
    .await
}