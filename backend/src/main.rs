mod global;
mod database;
mod storage;
mod collector;
mod batch_maker;
mod message_queue;
mod rules;
mod alert;
mod schema;

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use crate::collector::{LogCollector, ParseLogError, process_logs};
use crate::batch_maker::{create_batch};
use crate::alert::{get_alert, list_alerts, delete_alert, acknowledge_alert};
use serde_json::json;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn import_log_handler(collector: web::Data<LogCollector>, log: web::Json<String>, account_id: String) -> impl Responder {
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

async fn get_alert_handler(alert_id: web::Path<String>) -> impl Responder {
    match get_alert(&alert_id.to_string()) {
        Ok(alert) => HttpResponse::Ok().json(alert),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

async fn get_all_alerts_handler(account_id: web::Path<String>) -> impl Responder {
    match list_alerts(&account_id) {
        Ok(alerts) => HttpResponse::Ok().json(alerts),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

async fn delete_alert_handler(alert_id: web::Path<String>) -> impl Responder {
    match delete_alert(&alert_id.to_string()) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

async fn acknowledge_alert_handler(alert_id: web::Path<String>) -> impl Responder {
    match acknowledge_alert(&alert_id.to_string()) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
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
                    .route("/import_log", web::post().to(import_log_handler))
                    .service(
                        web::scope("/alert")
                            .route("/get_all/{alert_id}", web::get().to(get_alert_handler))
                            .route("/get_all/{account_id}", web::get().to(get_all_alerts_handler))
                            .route("/delete_alert/{alert_id}", web::delete().to(delete_alert_handler))
                            .route("/acknowledge_alert/{alert_id}", web::put().to(acknowledge_alert_handler))
                    )
            )
    })
    .bind(("127.0.0.1", 4200))?
    .run()
    .await
}