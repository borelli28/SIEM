use actix_web::{web, HttpRequest, HttpResponse, Responder, Error};
use serde_json::json;
use crate::alert::{get_alert, list_alerts, delete_alert, acknowledge_alert, get_alert_case};
use crate::csrf::{CsrfMiddleware, csrf_validator};

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

pub async fn delete_alert_handler(
    req: HttpRequest,
    alert_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match delete_alert(&alert_id.to_string()) {
        Ok(ok) => Ok(HttpResponse::Ok().json(ok)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

pub async fn acknowledge_alert_handler(
    req: HttpRequest,
    alert_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match acknowledge_alert(&alert_id.to_string()) {
        Ok(ok) => Ok(HttpResponse::Ok().json(ok)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

pub async fn get_alert_case_handler(
    req: HttpRequest,
    alert_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match get_alert_case(&alert_id.to_string()) {
        Ok(Some(case_id)) => Ok(HttpResponse::Ok().json(json!({
            "in_case": true,
            "case_id": case_id
        }))),
        Ok(None) => Ok(HttpResponse::Ok().json(json!({
            "in_case": false,
            "case_id": null
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}