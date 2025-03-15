use actix_multipart::form::{tempfile::TempFile, MultipartForm, text::Text};
use actix_web::{web, HttpResponse, HttpRequest, Responder, Error};
use serde::Deserialize;
use serde_json::json;
use crate::collector::{LogCollector, process_logs};
use crate::csrf::{CsrfMiddleware, csrf_validator};
use crate::log::{get_all_logs, get_query_logs};
use crate::batch_maker::create_batches;

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(rename = "file")]
    log_file: TempFile,
    account_id: Text<String>,
    host_id: Text<String>,
}

#[derive(Deserialize)]
pub struct QueryParams {
    pub account_id: String,
    pub query: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

pub async fn import_log_handler(
    req: HttpRequest,
    csrf: web::Data<CsrfMiddleware>,
    collector: web::Data<LogCollector>,
    form: MultipartForm<UploadForm>,
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    let UploadForm { log_file, account_id, host_id } = form.into_inner();
    let log_file_path = log_file.file.path();

    match create_batches(log_file_path.to_str().unwrap()).await {
        Ok(_) => {
            match process_logs(&collector, account_id.to_string(), host_id.to_string()).await {
                Ok(_) => Ok(HttpResponse::Ok().json(json!({ "status": "ok" }))),
                Err(err) => {
                    Ok(HttpResponse::InternalServerError().json(json!({
                        "status": "error",
                        "message": "Failed to process logs",
                        "details": err.to_string()
                    })))
                }
            }
        },
        Err(err) => Ok(HttpResponse::BadRequest().json(json!({
            "status": "error",
            "message": format!("Invalid log format: {:?}", err)
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

pub async fn get_query_logs_handler(
    query_params: web::Query<QueryParams>,
    _req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    if query_params.account_id.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Account ID is required"
        })));
    }

    match get_query_logs(&query_params.account_id, &query_params.query, query_params.start_time.clone(), query_params.end_time.clone()) {
        Ok(logs) => Ok(HttpResponse::Ok().json(logs)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })))
    }
}