use crate::auth_session::{verify_session, invalidate_session};
use actix_web::{HttpResponse, HttpRequest, Responder};
use actix_session::Session;

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