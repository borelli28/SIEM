use actix_web::{web, HttpResponse, HttpRequest, Error};
use crate::csrf::{CsrfMiddleware, csrf_validator};

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