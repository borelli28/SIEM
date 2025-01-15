use rusqlite::{Error as SqliteError, OptionalExtension, params};
use crate::database::establish_connection;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use serde_json;
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum CaseSeverity {
    Low,
    Medium,
    High,
}

impl fmt::Display for CaseSeverity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CaseSeverity::Low => write!(f, "Low"),
            CaseSeverity::Medium => write!(f, "Medium"),
            CaseSeverity::High => write!(f, "High"),
        }
    }
}

impl From<String> for CaseSeverity {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "low" => CaseSeverity::Low,
            "medium" => CaseSeverity::Medium,
            "high" => CaseSeverity::High,
            _ => CaseSeverity::Low,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum CaseStatus {
    InProgress,
    Closed,
    Hold,
}

impl fmt::Display for CaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CaseStatus::InProgress => write!(f, "InProgress"),
            CaseStatus::Closed => write!(f, "Closed"),
            CaseStatus::Hold => write!(f, "Hold"),
        }
    }
}

impl From<String> for CaseStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "inprogress" => CaseStatus::InProgress,
            "closed" => CaseStatus::Closed,
            "hold" => CaseStatus::Hold,
            _ => CaseStatus::InProgress,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Observable {
    pub observable_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Case {
    pub id: String,
    pub account_id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub category: String,
    pub analyst_assigned: String,
    pub observables: Vec<Observable>,
    pub comments: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for Case {
    fn default() -> Self {
        Case {
            id: Uuid::new_v4().to_string(),
            account_id: String::new(),
            title: "New Investigation".to_string(),
            description: "New security investigation case".to_string(),
            severity: "Low".to_string(),
            status: "InProgress".to_string(),
            category: "Security Investigation".to_string(),
            analyst_assigned: String::new(),
            observables: Vec::new(),
            comments: Vec::new(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug)]
pub enum CaseError {
    DatabaseError(SqliteError),
    ValidationError(String),
    SerializationError(serde_json::Error),
}

impl fmt::Display for CaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CaseError::DatabaseError(err) => write!(f, "Database error: {}", err),
            CaseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            CaseError::SerializationError(err) => write!(f, "Serialization error: {}", err),
        }
    }
}

impl From<SqliteError> for CaseError {
    fn from(err: SqliteError) -> Self {
        CaseError::DatabaseError(err)
    }
}

impl From<serde_json::Error> for CaseError {
    fn from(err: serde_json::Error) -> Self {
        CaseError::SerializationError(err)
    }
}

impl Case {
    fn validate(&self) -> Result<(), CaseError> {
        if self.account_id.is_empty() {
            return Err(CaseError::ValidationError("Account ID cannot be empty".to_string()));
        }
        if self.analyst_assigned.is_empty() {
            return Err(CaseError::ValidationError("Analyst must be assigned".to_string()));
        }
        let _ = CaseSeverity::from(self.severity.clone());
        let _ = CaseStatus::from(self.status.clone());
        Ok(())
    }

    pub fn new(account_id: String, analyst_assigned: String) -> Self {
        let mut default_case = Case::default();
        default_case.account_id = account_id;
        default_case.analyst_assigned = analyst_assigned;
        default_case
    }
}

pub fn create_case(account_id: &str, analyst_assigned: &str) -> Result<Case, CaseError> {
    let new_case = Case::new(account_id.to_string(), analyst_assigned.to_string());
    new_case.validate()?;

    let observables_json = serde_json::to_string(&new_case.observables)?;
    let comments_json = serde_json::to_string(&new_case.comments)?;

    let conn = establish_connection()?;
    conn.execute(
        "INSERT INTO cases (id, account_id, title, description, severity, status, category, 
         analyst_assigned, observables, comments, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            new_case.id,
            new_case.account_id,
            new_case.title,
            new_case.description,
            new_case.severity,
            new_case.status,
            new_case.category,
            new_case.analyst_assigned,
            observables_json,
            comments_json,
            new_case.created_at,
            new_case.updated_at,
        ],
    )?;

    Ok(new_case)
}

pub fn get_case(case_id: &str) -> Result<Option<Case>, CaseError> {
    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, account_id, title, description, severity, status, category, 
         analyst_assigned, observables, comments, created_at, updated_at 
         FROM cases WHERE id = ?1"
    )?;

    let case = stmt.query_row(params![case_id], |row| {
        let observables_json: String = row.get(8)?;
        let comments_json: String = row.get(9)?;
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
            observables_json,
            comments_json,
            row.get(10)?,
            row.get(11)?
        ))
    }).optional()?;

    match case {
        Some((id, account_id, title, description, severity, status, category, 
              analyst_assigned, observables_json, comments_json, created_at, updated_at)) => {
            let observables = serde_json::from_str(&observables_json)?;
            let comments = serde_json::from_str(&comments_json)?;
            Ok(Some(Case {
                id,
                account_id,
                title,
                description,
                severity,
                status,
                category,
                analyst_assigned,
                observables,
                comments,
                created_at,
                updated_at,
            }))
        }
        None => Ok(None)
    }
}

pub fn get_cases_by_account(account_id: &str) -> Result<Vec<Case>, CaseError> {
    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, account_id, title, description, severity, status, category, 
         analyst_assigned, observables, comments, created_at, updated_at 
         FROM cases WHERE account_id = ?1 
         ORDER BY created_at DESC"
    )?;

    let cases_iter = stmt.query_map(params![account_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, String>(9)?,
            row.get::<_, String>(10)?,
            row.get::<_, String>(11)?,
        ))
    })?;

    let cases_data: Result<Vec<_>, _> = cases_iter.collect();
    let cases_data = cases_data?;

    let mut cases = Vec::new();
    for (id, account_id, title, description, severity, status, category, 
         analyst_assigned, observables_json, comments_json, created_at, updated_at) in cases_data {
        let observables: Vec<Observable> = serde_json::from_str(&observables_json)?;
        let comments: Vec<String> = serde_json::from_str(&comments_json)?;
        
        cases.push(Case {
            id,
            account_id,
            title,
            description,
            severity,
            status,
            category,
            analyst_assigned,
            observables,
            comments,
            created_at,
            updated_at,
        });
    }

    Ok(cases)
}

pub fn update_case(case: &Case) -> Result<(), CaseError> {
    case.validate()?;
    let observables_json = serde_json::to_string(&case.observables)?;
    let comments_json = serde_json::to_string(&case.comments)?;

    let conn = establish_connection()?;
    conn.execute(
        "UPDATE cases 
         SET title = ?2, description = ?3, severity = ?4, status = ?5, 
         category = ?6, analyst_assigned = ?7, observables = ?8, comments = ?9, 
         updated_at = ?10 
         WHERE id = ?1",
        params![
            case.id,
            case.title,
            case.description,
            case.severity,
            case.status,
            case.category,
            case.analyst_assigned,
            observables_json,
            comments_json,
            Utc::now().to_rfc3339(),
        ],
    )?;

    Ok(())
}

pub fn delete_case(case_id: &str) -> Result<bool, CaseError> {
    let conn = establish_connection()?;
    let affected_rows = conn.execute(
        "DELETE FROM cases WHERE id = ?1",
        params![case_id],
    )?;

    Ok(affected_rows > 0)
}

pub fn add_observable(case_id: &str, observable: Observable) -> Result<(), CaseError> {
    let mut case = get_case(case_id)?.ok_or_else(|| {
        CaseError::ValidationError("Case not found".to_string())
    })?;

    case.observables.push(observable);
    update_case(&case)?;

    Ok(())
}

pub fn add_comment(case_id: &str, comment: String) -> Result<(), CaseError> {
    let mut case = get_case(case_id)?.ok_or_else(|| {
        CaseError::ValidationError("Case not found".to_string())
    })?;

    case.comments.push(comment);
    update_case(&case)?;

    Ok(())
}