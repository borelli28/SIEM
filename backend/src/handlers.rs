use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use crate::collector::{LogCollector, ParseLogError, process_logs};
use crate::batch_maker::create_batch;
use crate::alert::{get_alert, list_alerts, delete_alert, acknowledge_alert};
use crate::host::{Host, create_host, get_host, get_all_hosts, update_host, delete_host};
use crate::rules::{AlertRule, create_rule, get_rule, list_rules, update_rule, delete_rule};
use crate::log::{get_all_logs};
use crate::account::{Account, AccountError, create_account, get_account, update_account, delete_account, verify_login};

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

// Logs Handlers
//
pub async fn import_log_handler(
    collector: web::Data<LogCollector>, 
    log: web::Json<String>, 
    account_id: web::Json<String>, 
    host_id: web::Json<String>
    ) -> impl Responder {
    match create_batch(&log.into_inner()).await {
        Ok(_) => {
            match process_logs(&collector, account_id.to_string(), host_id.to_string()).await {
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

pub async fn get_logs_handler(account_id: web::Path<String>) -> impl Responder {
    match get_all_logs(&account_id) {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

// Alert Handlers
//
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

// Host Handlers
//
pub async fn create_host_handler(host: web::Json<Host>, account_id: web::Path<String>) -> impl Responder {
    match create_host(&host, &account_id.to_string()) {
        Ok(host) => HttpResponse::Ok().json(host),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
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

pub async fn edit_host_handler(host: web::Json<Host>) -> impl Responder {
    match update_host(&host) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn delete_host_handler(host_id: web::Path<String>) -> impl Responder {
    match delete_host(&host_id.to_string()) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

// Host Handlers
//
pub async fn create_rule_handler(rule: web::Json<AlertRule>) -> impl Responder {
    match create_rule(&rule) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
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

pub async fn edit_rule_handler(rule: web::Json<AlertRule>) -> impl Responder {
    match update_rule(&rule) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn delete_rule_handler(rule_id: web::Path<String>) -> impl Responder {
    match delete_rule(&rule_id.to_string()) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

// Account Handlers
//
pub async fn create_account_handler(account: web::Json<Account>) -> impl Responder {
    let account = account.into_inner();
    let name = account.name;
    let password = account.password;
    let role = account.role;

    match create_account(name, password, role) {
        Ok(account) => HttpResponse::Ok().json(account),
        Err(err) => match err {
            AccountError::InvalidRole => HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Invalid role provided"
            })),
            AccountError::ExpectedField(field) => HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": format!("Missing required field: {}", field)
            })),
            _ => HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            }))
        }
    }
}

pub async fn get_account_handler(account_id: web::Path<String>) -> impl Responder {
    match get_account(&account_id.to_string()) {
        Ok(host) => HttpResponse::Ok().json(host),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn edit_account_handler(account: web::Json<Account>) -> impl Responder {
    match update_account(&account) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn delete_account_handler(account_id: web::Path<String>) -> impl Responder {
    match delete_account(&account_id.to_string()) {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        }))
    }
}

pub async fn login_account_handler(account: web::Json<Account>) -> impl Responder {
    let account = account.into_inner();
    let name = account.name;
    let password = account.password;

    match verify_login(&name, &password) {
        Ok(account) => HttpResponse::Ok().json(account),
        Err(err) => match err {
            AccountError::InvalidRole => HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Invalid role provided"
            })),
            AccountError::ExpectedField(field) => HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": format!("Missing required field: {}", field)
            })),
            _ => HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            }))
        }
    }
}