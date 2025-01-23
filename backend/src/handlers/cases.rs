use actix_web::{web, HttpResponse, HttpRequest, Error};
use serde_json::json;
use log::error;
use crate::cases::{Case, CaseError, Observable, create_case, get_case, get_cases_by_account,
                   update_case, delete_case, add_observable};
use crate::case_comments::{CaseCommentError, create_comment,
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

pub async fn add_comment_handler(
    req: HttpRequest,
    case_id: web::Path<String>,
    comment: web::Json<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    match create_comment(&case_id, &comment.into_inner()) {
        Ok(new_comment) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Comment added successfully",
            "data": {
                "id": new_comment.id,
                "case_id": new_comment.case_id,
                "comment": new_comment.comment,
                "created_at": new_comment.created_at,
                "updated_at": new_comment.updated_at
            }
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

pub async fn update_comment_handler(
    req: HttpRequest,
    comment_id: web::Path<String>,
    comment_text: web::Json<String>,
    csrf: web::Data<CsrfMiddleware>
) -> Result<HttpResponse, Error> {
    csrf_validator(&req, &csrf).await?;

    match update_comment(&comment_id, &comment_text.into_inner()) {
        Ok(updated_comment) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Comment updated successfully",
            "data": {
                "id": updated_comment.id,
                "case_id": updated_comment.case_id,
                "comment": updated_comment.comment,
                "created_at": updated_comment.created_at,
                "updated_at": updated_comment.updated_at
            }
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
}