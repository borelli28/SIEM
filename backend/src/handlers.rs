use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use crate::collector::{LogCollector, ParseLogError, process_logs};
use crate::batch_maker::create_batch;
use crate::alert::{get_alert, list_alerts, delete_alert, acknowledge_alert};

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

pub async fn import_log_handler(collector: web::Data<LogCollector>, log: web::Json<String>, account_id: String) -> impl Responder {
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

pub async fn get_alert_handler(alert_id: web::Path<String>) -> impl Responder {
    match get_alert(&alert_id.to_string()) {
        Ok(alert) => HttpResponse::Ok().json(alert),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn get_all_alerts_handler(account_id: web::Path<String>) -> impl Responder {
    match list_alerts(&account_id) {
        Ok(alerts) => HttpResponse::Ok().json(alerts),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn delete_alert_handler(alert_id: web::Path<String>) -> impl Responder {
    match delete_alert(&alert_id.to_string()) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn acknowledge_alert_handler(alert_id: web::Path<String>) -> impl Responder {
    match acknowledge_alert(&alert_id.to_string()) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}