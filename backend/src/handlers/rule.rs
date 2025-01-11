use actix_web::{web, HttpResponse, HttpRequest, Responder, Error};
use serde_json::json;
use log::info;
use crate::rules::{Rule, create_rule, get_rule, list_rules, update_rule, delete_rule};
use crate::csrf::{CsrfMiddleware, csrf_validator};

pub async fn create_rule_handler(
    req: HttpRequest,
    rule: web::Json<Rule>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    info!("create_rule_handler()");
    info!("Rule date: {:?}", rule.date);
    info!("created date: {:?}", rule.created_at);
    info!("updated date: {:?}", rule.updated_at);
    match create_rule(&rule) {
        Ok(ok) => Ok(HttpResponse::Ok().json(ok)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

pub async fn get_rule_handler(rule_id: web::Path<String>) -> impl Responder {
    match get_rule(&rule_id.to_string()) {
        Ok(rule) => HttpResponse::Ok().json(rule),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn get_all_rules_handler(account_id: web::Path<String>) -> impl Responder {
    match list_rules(&account_id.to_string()) {
        Ok(rules) => HttpResponse::Ok().json(rules),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn edit_rule_handler(
    req: HttpRequest,
    rule: web::Json<Rule>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match update_rule(&rule) {
        Ok(ok) => Ok(HttpResponse::Ok().json(ok)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

pub async fn delete_rule_handler(
    req: HttpRequest,
    rule_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match delete_rule(&rule_id.to_string()) {
        Ok(ok) => Ok(HttpResponse::Ok().json(ok)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}