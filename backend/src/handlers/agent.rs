use actix_multipart::form::{tempfile::TempFile, MultipartForm, text::Text};
use actix_web::{web, HttpResponse, Error};
use serde::Deserialize;
use serde_json::json;
use chrono::Utc;
use crate::agent::{Agent, register_agent, verify_agent_api_key, update_agent_last_seen};
use crate::collector::{LogCollector, process_logs};
use crate::batch_maker::create_batches;

#[derive(Debug, MultipartForm)]
pub struct AgentUploadForm {
    #[multipart(rename = "file")]
    log_file: TempFile,
    api_key: Text<String>,
    account_id: Text<String>,
    host_id: Text<String>,
}

pub async fn register_agent_handler(
    agent: web::Json<Agent>
) -> Result<HttpResponse, Error> {
    match register_agent(&agent) {
        Ok((id, api_key)) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "agent_id": id,
            "api_key": api_key
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

#[derive(Deserialize)]
pub struct HeartbeatRequest {
    api_key: String,
}

pub async fn agent_heartbeat_handler(
    payload: web::Json<HeartbeatRequest>,
) -> Result<HttpResponse, Error> {
    match verify_agent_api_key(&payload.api_key) {
        Ok(true) => {
            match update_agent_last_seen(&payload.api_key) {
                Ok(_) => Ok(HttpResponse::Ok().json(json!({
                    "status": "success",
                    "timestamp": Utc::now()
                }))),
                Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": err.to_string()
                })))
            }
        },
        Ok(false) => Ok(HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid API key"
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}

pub async fn agent_upload_handler(
    form: MultipartForm<AgentUploadForm>,
    collector: web::Data<LogCollector>,
) -> Result<HttpResponse, Error> {
    let AgentUploadForm { log_file, api_key, account_id, host_id } = form.into_inner();
    let log_file_path = log_file.file.path();

    match verify_agent_api_key(&api_key) {
        Ok(true) => {
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
        },
        Ok(false) => Ok(HttpResponse::Unauthorized().json(json!({
            "status": "error",
            "message": "Invalid API key"
        }))),
        Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": err.to_string()
        })))
    }
}