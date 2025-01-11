use actix_web::{web, HttpResponse, HttpRequest, Responder, Error};
use serde_json::json;
use crate::host::{Host, create_host, get_host, get_all_hosts, update_host, delete_host};
use crate::csrf::{CsrfMiddleware, csrf_validator};

pub async fn create_host_handler(
    req: HttpRequest,
    host: web::Json<Host>,
    account_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match create_host(&host, &account_id.to_string()) {
        Ok(host) => Ok(HttpResponse::Ok().json(host)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

pub async fn get_host_handler(host_id: web::Path<String>) -> impl Responder {
    match get_host(&host_id.to_string()) {
        Ok(host) => HttpResponse::Ok().json(host),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn get_all_hosts_handler(account_id: web::Path<String>) -> impl Responder {
    match get_all_hosts(&account_id.to_string()) {
        Ok(host) => HttpResponse::Ok().json(host),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn edit_host_handler(
    req: HttpRequest,
    host: web::Json<Host>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match update_host(&host) {
        Ok(ok) => Ok(HttpResponse::Ok().json(ok)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

pub async fn delete_host_handler(
    req: HttpRequest,
    host_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match delete_host(&host_id.to_string()) {
        Ok(ok) => Ok(HttpResponse::Ok().json(ok)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}