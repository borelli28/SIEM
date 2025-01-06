use chrono::NaiveDateTime;
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
}

impl fmt::Display for EqlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EqlError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            EqlError::QueryBuildError(msg) => write!(f, "Query build error: {}", msg),
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

pub struct QueryBuilder;

impl QueryBuilder {
    // Converts tokens into a SQL query
    pub fn build_query(tokens: Vec<Token>) -> Result<(String, Vec<String>), EqlError> {
        let mut query = String::from("SELECT * FROM logs WHERE ");
        let mut params = Vec::new();
        let mut current_condition = String::new();

        for token in tokens {
            match token {
                Token::Field(field) => {
                    current_condition = field;
                }
                Token::Operator(op) => {
                    match op.as_str() {
                        "=" => current_condition.push_str(" = ?"),
                        "!=" => current_condition.push_str(" != ?"),
                        ">" => current_condition.push_str(" > ?"),
                        "<" => current_condition.push_str(" < ?"),
                        _ => return Err(EqlError::QueryBuildError("Invalid operator".to_string())),
                    }
                }
                Token::Value(val) => {
                    params.push(val);
                    query.push_str(&current_condition);
                    current_condition.clear();
                }
                Token::And => query.push_str(" AND "),
                Token::Or => query.push_str(" OR "),
                Token::TimeRange(range) => {
                    // Handle time range parsing
                    let (time_query, time_param) = Self::parse_time_range(&range)?;
                    query.push_str(&time_query);
                    params.push(time_param);
                }
                _ => {}
            }
        }

        Ok((query, params))
    }
 
    // Helper function to parse time range specifications
    fn parse_time_range(range: &str) -> Result<(String, String), EqlError> {
        // Example format: timestamp>2023-01-01
        let mut parts = range.split(|c| c == '>' || c == '<');
        let field = parts.next().ok_or_else(|| EqlError::QueryBuildError("Invalid time range".to_string()))?;
        let value = parts.next().ok_or_else(|| EqlError::QueryBuildError("Invalid time range".to_string()))?;

        // Validate the datetime format
        if NaiveDateTime::parse_from_str(value, "%Y-%m-%d").is_err() {
            return Err(EqlError::QueryBuildError("Invalid datetime format".to_string()));
        }

        Ok((format!("{} > ?", field), value.to_string()))
    }
}