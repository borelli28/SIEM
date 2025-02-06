use actix_web::{web, HttpResponse, HttpRequest, Responder, Error};
use actix_session::Session;
use serde_json::json;
use log::error;
use crate::account::{Account, AccountError, create_account, get_account, update_account, delete_account, verify_login};
use crate::csrf::{CsrfMiddleware, csrf_validator};

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
            AccountError::ValidationError(error) => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": error.to_string()
            }))),
            _ => {
                error!("Internal server error: {:?}", err);
                Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "An internal error occurred"
                })))
            }
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