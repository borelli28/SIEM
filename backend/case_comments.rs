use rusqlite::{Error as SqliteError, OptionalExtension, params};
use crate::database::establish_connection;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct CaseComment {
    pub id: String,
    pub case_id: String,
    pub comment: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for CaseComment {
    fn default() -> Self {
        CaseComment {
            id: Uuid::new_v4().to_string(),
            case_id: String::new(),
            comment: String::new(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug)]
pub enum CaseCommentError {
    DatabaseError(SqliteError),
    ValidationError(String),
}

impl fmt::Display for CaseCommentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaseCommentError::DatabaseError(err) => write!(f, "Database error: {}", err),
            CaseCommentError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl From<SqliteError> for CaseCommentError {
    fn from(err: SqliteError) -> Self {
        CaseCommentError::DatabaseError(err)
    }
}

pub fn create_comment(case_id: &str, comment_text: &str) -> Result<CaseComment, CaseCommentError> {
    if comment_text.is_empty() {
        return Err(CaseCommentError::ValidationError("Comment cannot be empty".to_string()));
    }

    let comment = CaseComment {
        id: Uuid::new_v4().to_string(),
        case_id: case_id.to_string(),
        comment: comment_text.to_string(),
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };

    let conn = establish_connection()?;
    conn.execute(
        "INSERT INTO case_comments (id, case_id, comment, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            comment.id,
            comment.case_id,
            comment.comment,
            comment.created_at,
            comment.updated_at,
        ],
    )?;

    Ok(comment)
}

pub fn get_comment(comment_id: &str) -> Result<Option<CaseComment>, CaseCommentError> {
    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, case_id, comment, created_at, updated_at 
         FROM case_comments WHERE id = ?1"
    )?;

    let comment = stmt.query_row(params![comment_id], |row| {
        Ok(CaseComment {
            id: row.get(0)?,
            case_id: row.get(1)?,
            comment: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }).optional()?;

    Ok(comment)
}

pub fn get_comments_by_case(case_id: &str) -> Result<Vec<CaseComment>, CaseCommentError> {
    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, case_id, comment, created_at, updated_at 
         FROM case_comments 
         WHERE case_id = ?1 
         ORDER BY created_at DESC"
    )?;

    let comments_iter = stmt.query_map(params![case_id], |row| {
        Ok(CaseComment {
            id: row.get(0)?,
            case_id: row.get(1)?,
            comment: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;

    let comments: Result<Vec<_>, _> = comments_iter.collect();
    Ok(comments?)
}

pub fn update_comment(comment: &CaseComment) -> Result<(), CaseCommentError> {
    if comment.comment.is_empty() {
        return Err(CaseCommentError::ValidationError("Comment cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    conn.execute(
        "UPDATE case_comments 
         SET comment = ?2, updated_at = ?3 
         WHERE id = ?1",
        params![
            comment.id,
            comment.comment,
            Utc::now().to_rfc3339(),
        ],
    )?;

    Ok(())
}

pub fn delete_comment(comment_id: &str) -> Result<bool, CaseCommentError> {
    let conn = establish_connection()?;
    let affected_rows = conn.execute(
        "DELETE FROM case_comments WHERE id = ?1",
        params![comment_id],
    )?;

    Ok(affected_rows > 0)
}