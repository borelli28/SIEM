use actix_web::{web, HttpResponse, HttpRequest, Error};
use serde_json::json;
use chrono::Utc;
use log::error;
use crate::cases::{Case, CaseError, Observable, create_case, get_case, get_cases_by_account,
                   update_case, delete_case, add_observable, delete_observable};
use crate::case_comments::{CaseCommentError, create_comment, get_comment,
    get_comments_by_case, update_comment, delete_comment};
use crate::csrf::{CsrfMiddleware, csrf_validator};

pub async fn create_case_handler(
    req: HttpRequest,
    account_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    match create_case(&account_id) {
        Ok(case) => Ok(HttpResponse::Ok().json(case)),
        Err(err) => match err {
            CaseError::ValidationError(msg) => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": msg
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

pub async fn get_case_handler(
    case_id: web::Path<String>
) -> Result<HttpResponse, Error> {
    match get_case(&case_id) {
        Ok(Some(case)) => Ok(HttpResponse::Ok().json(case)),
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Case not found"
        }))),
        Err(err) => {
            error!("Internal server error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            })))
        }
    }
}

pub async fn get_cases_by_account_handler(
    account_id: web::Path<String>
) -> Result<HttpResponse, Error> {
    match get_cases_by_account(&account_id) {
        Ok(cases) => Ok(HttpResponse::Ok().json(cases)),
        Err(err) => {
            error!("Internal server error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            })))
        }
    }
}

pub async fn update_case_handler(
    req: HttpRequest,
    case_data: web::Json<Case>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    
    match update_case(&case_data) {
        Ok(()) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Case updated successfully"
        }))),
        Err(err) => match err {
            CaseError::ValidationError(msg) => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": msg
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

pub async fn delete_case_handler(
    req: HttpRequest,
    case_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    match delete_case(&case_id) {
        Ok(true) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Case deleted successfully"
        }))),
        Ok(false) => Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Case not found"
        }))),
        Err(err) => {
            error!("Internal server error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            })))
        }
    }
}

pub async fn add_observable_handler(
    req: HttpRequest,
    case_id: web::Path<String>,
    observable: web::Json<Observable>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    match add_observable(&case_id, observable.into_inner()) {
        Ok(()) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Observable added successfully"
        }))),
        Err(err) => match err {
            CaseError::ValidationError(msg) => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": msg
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

pub async fn delete_observable_handler(
    req: HttpRequest,
    case_id: web::Path<String>,
    observable: web::Json<Observable>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    match delete_observable(&case_id, observable.into_inner()) {
        Ok(()) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Observable deleted successfully"
        }))),
        Err(err) => match err {
            CaseError::ValidationError(msg) => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": msg
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

pub async fn add_comment_handler(
    req: HttpRequest,
    case_id: web::Path<String>,
    comment: web::Json<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    match create_comment(&case_id, &comment.into_inner()) {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Comment added successfully"
        }))),
        Err(err) => match err {
            CaseCommentError::ValidationError(msg) => Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": msg
            }))),
            CaseCommentError::DatabaseError(e) => {
                error!("Database error while adding comment: {:?}", e);
                Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "An internal error occurred"
                })))
            }
        }
    }
}

pub async fn get_case_comments_handler(
    case_id: web::Path<String>
) -> Result<HttpResponse, Error> {
    match get_comments_by_case(&case_id) {
        Ok(comments) => Ok(HttpResponse::Ok().json(comments)),
        Err(err) => {
            error!("Internal server error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            })))
        }
    }
}

pub async fn update_comment_handler(
    req: HttpRequest,
    comment_id: web::Path<String>,
    comment_text: web::Json<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    // Get the existing comment
    match get_comment(&comment_id) {
        Ok(Some(mut comment)) => {
            // Update the comment text
            comment.comment = comment_text.into_inner();
            comment.updated_at = Utc::now().to_rfc3339();

            // Save the updated comment
            match update_comment(&comment) {
                Ok(()) => Ok(HttpResponse::Ok().json(json!({
                    "status": "success",
                    "message": "Comment updated successfully"
                }))),
                Err(err) => match err {
                    CaseCommentError::ValidationError(msg) => Ok(HttpResponse::BadRequest().json(json!({
                        "status": "error",
                        "message": msg
                    }))),
                    CaseCommentError::DatabaseError(e) => {
                        error!("Database error while updating comment: {:?}", e);
                        Ok(HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": "An internal error occurred"
                        })))
                    }
                }
            }
        },
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Comment not found"
        }))),
        Err(err) => {
            error!("Database error while fetching comment: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            })))
        }
    }
}

pub async fn delete_comment_handler(
    req: HttpRequest,
    comment_id: web::Path<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;
    
    match delete_comment(&comment_id) {
        Ok(true) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Comment deleted successfully"
        }))),
        Ok(false) => Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Comment not found"
        }))),
        Err(err) => {
            error!("Internal server error: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": "An internal error occurred"
            })))
        }
    }
}