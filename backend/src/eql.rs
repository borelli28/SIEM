use crate::database::establish_connection;
use serde_json::{Value, from_str};
use chrono::NaiveDateTime;
use rusqlite::params;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Field(String),      // Field names (e.g., severity, device_vendor)
    Value(String),      // Values (e.g., "high", "cisco")
    Operator(String),   // Operators (=, !=, >, <, etc.)
    And,                // Logical AND
    Or,                 // Logical OR
    OpenParen,          // (
    CloseParen,        // )
    Where,             // WHERE keyword
    TimeRange(String), // Time range specifications
}

#[derive(Debug)]
pub enum EqlError {
    ParseError(String),
    QueryBuildError(String),
    DatabaseError(String),
}

impl fmt::Display for EqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EqlError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            EqlError::QueryBuildError(msg) => write!(f, "Query build error: {}", msg),
            EqlError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

pub struct EqlParser;

impl EqlParser {
    // Get tokens from query
    pub fn parse(query: &str) -> Result<Vec<Token>, EqlError> {
        let mut tokens = Vec::new();
        let mut chars = query.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                // Skip whitespace
                c if c.is_whitespace() => {
                    chars.next();
                }

                // Handle alphabetic characters (fields, keywords)
                c if c.is_alphabetic() => {
                    let word = Self::read_word(&mut chars);
                    match word.to_lowercase().as_str() {
                        "where" => tokens.push(Token::Where),
                        "and" => tokens.push(Token::And),
                        "or" => tokens.push(Token::Or),
                        // If it's not a keyword, it's a field name
                        _ => tokens.push(Token::Field(word)),
                    }
                }

                // Handle operators
                '=' | '!' | '>' | '<' => {
                    let op = Self::read_operator(&mut chars);
                    tokens.push(Token::Operator(op));
                }

                // Handle string literals
                '"' => {
                    chars.next(); // Skip the opening quote
                    let value = Self::read_until_quote(&mut chars)?;
                    tokens.push(Token::Value(value));
                }

                // Handle parentheses
                '(' => {
                    chars.next();
                    tokens.push(Token::OpenParen);
                }
                ')' => {
                    chars.next();
                    tokens.push(Token::CloseParen);
                }

                // Handle time ranges (special syntax: @timestamp[>2023-01-01])
                '@' => {
                    chars.next();
                    let time_range = Self::read_time_range(&mut chars)?;
                    tokens.push(Token::TimeRange(time_range));
                }

                // Unexpected character
                _ => return Err(EqlError::ParseError(format!("Unexpected character: {}", c))),
            }
        }

        Ok(tokens)
    }

    // Lexer for words (field names or keywords)
    fn read_word(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut word = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                word.push(c);
                chars.next();
            } else {
                break;
            }
        }
        word
    }

    // Lexer for operators
    fn read_operator(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut op = String::new();
        while let Some(&c) = chars.peek() {
            if "=!><".contains(c) {
                op.push(c);
                chars.next();
            } else {
                break;
            }
        }
        op
    }

    // Lexer for quoted strings
    fn read_until_quote(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, EqlError> {
        let mut value = String::new();
        while let Some(c) = chars.next() {
            if c == '"' {
                return Ok(value);
            }
            value.push(c);
        }
        Err(EqlError::ParseError("Unterminated string literal".to_string()))
    }

    // Lexer for time range specifications
    fn read_time_range(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String, EqlError> {
        let mut range = String::new();
        while let Some(&c) = chars.peek() {
            if c == ']' {
                chars.next();
                return Ok(range);
            }
            range.push(c);
            chars.next();
        }
        Err(EqlError::ParseError("Unterminated time range".to_string()))
    }
}

#[derive(Debug, Clone)]
struct Condition {
    field: String,
    operator: String,
    value: String,
}

#[derive(Debug)]
struct EqlQuery {
    conditions: Vec<Condition>,
    time_range: Option<(String, String, String)>,
}

pub struct QueryExecutor;

impl QueryExecutor {
    // Parse EQL query into a structured format
    fn parse_query(tokens: Vec<Token>) -> Result<EqlQuery, EqlError> {
        let mut conditions = Vec::new();
        let mut time_range = None;
        let mut current_field = None;
        let mut current_operator = None;

        for token in tokens {
            match token {
                Token::Field(field) => {
                    if current_field.is_some() {
                        return Err(EqlError::ParseError("Unexpected field token".to_string()));
                    }
                    current_field = Some(field);
                }
                Token::Operator(op) => {
                    if current_operator.is_some() || current_field.is_none() {
                        return Err(EqlError::ParseError("Unexpected operator token".to_string()));
                    }
                    current_operator = Some(op);
                }
                Token::Value(val) => {
                    if let (Some(field), Some(operator)) = (current_field.take(), current_operator.take()) {
                        conditions.push(Condition { field, operator, value: val });
                    } else {
                        return Err(EqlError::ParseError("Value token without field or operator".to_string()));
                    }
                }
                Token::TimeRange(range) => {
                    let mut parts = range.split(|c| c == '>' || c == '<');
                    let field = parts.next().ok_or_else(|| EqlError::QueryBuildError("Invalid time range".to_string()))?;
                    let value = parts.next().ok_or_else(|| EqlError::QueryBuildError("Invalid time range".to_string()))?;
                    let operator = if range.contains('>') { ">" } else { "<" }.to_string();
                    if NaiveDateTime::parse_from_str(value, "%Y-%m-%d").is_err() {
                        return Err(EqlError::QueryBuildError("Invalid datetime format".to_string()));
                    }
                    time_range = Some((field.to_string(), operator, value.to_string()));
                }
                Token::And => {},
                Token::Or | Token::OpenParen | Token::CloseParen => {
                    return Err(EqlError::ParseError("Complex queries with OR or parentheses not yet supported".to_string()));
                }
                Token::Where => {},
            }
        }

        Ok(EqlQuery { conditions, time_range })
    }

    // Check if a log matches the EQL query
    fn matches_query(log_data: &str, query: &EqlQuery) -> Result<bool, EqlError> {
        let json: Value = from_str(log_data)
            .map_err(|e| EqlError::ParseError(format!("Failed to parse log_data JSON: {}", e)))?;

        // Check all conditions
        for condition in &query.conditions {
            let value = Self::get_json_value(&json, &condition.field);
            let matches = match condition.operator.as_str() {
                "=" => value == condition.value,
                "!=" => value != condition.value,
                ">" => value > condition.value,
                "<" => value < condition.value,
                _ => return Err(EqlError::QueryBuildError(format!("Unsupported operator: {}", condition.operator))),
            };
            if !matches {
                return Ok(false);
            }
        }

        Ok(true)
    }

    // Extract a value from JSON based on field path
    fn get_json_value(json: &Value, field: &str) -> String {
        match field {
            "timestamp" => json.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            "src_ip" => json.get("src_ip").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            "dst_ip" => json.get("dst_ip").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            "event_type" => json.get("event_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            _ => json.get("extensions")
                .and_then(|ext| ext.get(field))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }

    // Execute EQL query and return matching logs one at a time
    pub fn execute_query(
        account_id: &str,
        start_time: &str,
        end_time: &str,
        eql_query: &str,
    ) -> Result<Vec<String>, EqlError> {
        let conn = establish_connection()
            .map_err(|e| EqlError::DatabaseError(e.to_string()))?;

        // Parse the EQL query into a structured format
        let tokens = EqlParser::parse(eql_query)?;
        let query = Self::parse_query(tokens)?;

        // Prepare the base SQL query for filtering by account_id and timestamp
        let mut stmt = conn.prepare(
            "SELECT log_data FROM logs WHERE account_id = ?1 AND timestamp BETWEEN ?2 AND ?3"
        ).map_err(|e| EqlError::DatabaseError(e.to_string()))?;

        // Stream results one row at a time. We only load one log to memory at a time
        let rows = stmt.query_map(
            params![account_id, start_time, end_time],
            |row| row.get::<_, String>(0) // Only fetch log_data
        ).map_err(|e| EqlError::DatabaseError(e.to_string()))?;

        let mut matching_logs = Vec::new();

        // Process each log one at a time
        for row in rows {
            let log_data = row.map_err(|e| EqlError::DatabaseError(e.to_string()))?;
            if Self::matches_query(&log_data, &query)? {
                matching_logs.push(log_data);
            }
        }

        Ok(matching_logs)
    }
}