use actix_web::{web, HttpResponse, HttpRequest, Responder, Error};
use actix_session::Session;
use serde_json::json;
use crate::collector::{LogCollector, ParseLogError, process_logs};
use crate::batch_maker::create_batch;
use crate::alert::{get_alert, list_alerts, delete_alert, acknowledge_alert};
use crate::host::{Host, create_host, get_host, get_all_hosts, update_host, delete_host};
use crate::rules::{AlertRule, create_rule, get_rule, list_rules, update_rule, delete_rule};
use crate::log::{get_all_logs};
use crate::account::{Account, AccountError, create_account, get_account, update_account, delete_account, verify_login};
use crate::auth_session::{verify_session, invalidate_session};
use crate::csrf::{CsrfMiddleware, csrf_validator};

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

pub async fn verify_session_handler(session: Session, req: HttpRequest) -> impl Responder {
    match verify_session(&session, &req) {
        Ok(account_id) => HttpResponse::Ok().json(serde_json::json!({ "authenticated": true, "account_id": account_id })),
        Err(_) => HttpResponse::Unauthorized().json(serde_json::json!({ "authenticated": false, "message": "Not authenticated" }))
    }
}

pub async fn logout_handler(session: Session) -> impl Responder {
    invalidate_session(&session);
    HttpResponse::Ok().json(serde_json::json!({ "message": "Logged out successfully" }))
}

// Logs Handlers
//
pub async fn import_log_handler(
    req: HttpRequest,
    csrf: web::Data<CsrfMiddleware>,
    collector: web::Data<LogCollector>,
    log: web::Json<String>,
    account_id: web::Json<String>,
    host_id: web::Json<String>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    
    match create_batch(&log.into_inner()).await {
        Ok(_) => {
            match process_logs(&collector, account_id.to_string(), host_id.to_string()).await {
                Ok(_) => Ok(HttpResponse::Ok().json(json!({ "status": "ok" }))),
                Err(e) => {
                    eprintln!("Error processing logs: {}", e);
                    match e {
                        ParseLogError::DatabaseError(msg) => {
                            Ok(HttpResponse::InternalServerError().json(json!({
                                "status": "error",
                                "message": "Database error",
                                "details": msg
                            })))
                        },
                        _ => Ok(HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "Failed to process logs"
                        })))
                    }
                }
            }
        },
        Err(_) => Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": "Invalid log format"
        }))),
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

// Host Handlers
//
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

// Rule Handlers
//
pub async fn create_rule_handler(
    req: HttpRequest,
    rule: web::Json<AlertRule>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
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
    rule: web::Json<AlertRule>,
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

// Account Handlers
//
pub async fn create_account_handler(
    req: HttpRequest,
    account: web::Json<Account>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    let account = account.into_inner();
    let name = account.name;
    let password = account.password;
    let role = account.role;
    match create_account(name, password, role) {
        Ok(account) => Ok(HttpResponse::Ok().json(account)),
        Err(err) => match err {
            AccountError::InvalidRole => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Invalid role provided"
            }))),
            AccountError::ExpectedField(field) => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": format!("Missing required field: {}", field)
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            })))
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

pub async fn edit_account_handler(
    req: HttpRequest,
    account: web::Json<Account>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match update_account(&account) {
        Ok(ok) => Ok(HttpResponse::Ok().json(ok)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

pub async fn delete_account_handler(
    req: HttpRequest,
    account_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    match delete_account(&account_id.to_string()) {
        Ok(ok) => Ok(HttpResponse::Ok().json(ok)),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

pub async fn login_account_handler(
    req: HttpRequest,
    session: Session,
    account: web::Json<Account>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    let account_data = account.into_inner();
    let name = account_data.name;
    let password = account_data.password;
    match verify_login(&session, &name, &password, &req) {
        Ok(Some(account)) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Login successful!",
            "account": account
        }))),
        Ok(None) => Ok(HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid username or password"
        }))),
        Err(err) => match err {
            AccountError::InvalidRole => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "Invalid role provided"
            }))),
            AccountError::ExpectedField(field) => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": format!("Missing required field: {}", field)
            }))),
            _ => Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            }))),
        },
    }
}

// CSRF Handlers
//
pub async fn get_csrf_handler(req: HttpRequest, csrf: web::Data<CsrfMiddleware>) -> HttpResponse {
    let form_id = req.headers()
        .get("X-Form-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default");

    match csrf.generate_token_pair(form_id) {
        Ok((token, cookie)) => {
            HttpResponse::Ok()
                .cookie(cookie)
                .json(token)
        },
        Err(_) => {
            HttpResponse::InternalServerError().finish()
        },
    }
}

pub async fn csrf_validator_handler(req: HttpRequest, csrf: web::Data<CsrfMiddleware>) -> Result<HttpResponse, Error> {
    match csrf_validator(&req, &csrf).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()), // CSRF validation passed
        Err(e) => Ok(HttpResponse::Forbidden().body(e.to_string())), // CSRF validation failed
    }
}