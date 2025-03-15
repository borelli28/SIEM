use serde::{Serialize, Deserialize};
use serde_json::{self, Value};
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug)]
pub enum ParseLogError {
    InvalidCEFFormat,
    InvalidFormat(String),
    SerializationError(String),
    DatabaseError(String),
}

impl std::fmt::Display for ParseLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseLogError::InvalidCEFFormat => write!(f, "Invalid CEF format"),
            ParseLogError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseLogError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ParseLogError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for ParseLogError {}

#[derive(Serialize, Deserialize, Debug)]
pub struct NormalizedLog {
    pub timestamp: Option<String>,
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub event_type: Option<String>,
    pub host_id: String,
    pub account_id: String,
    pub raw: String,
    pub extensions: HashMap<String, String>,
}

fn clean_log(log: &str) -> String {
    log.trim()
        .lines()
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join(" ")
}

enum LogFormat {
    Cef,
    Syslog,
    Json,
    Unknown,
}

fn detect_format(log: &str) -> LogFormat {
    let cleaned = clean_log(log);
    if cleaned.starts_with("CEF:") {
        LogFormat::Cef
    } else if cleaned.starts_with('<') && cleaned.contains('>') && cleaned[1..].chars().next().unwrap().is_digit(10) {
        // Check for <priority> followed by timestamp-like pattern
        let after_priority = cleaned.splitn(2, '>').nth(1).unwrap_or("");
        if after_priority.len() > 11 && after_priority[3..6].contains(" ") && after_priority[6..11].contains(":") {
            LogFormat::Syslog
        } else {
            LogFormat::Unknown
        }
    } else if cleaned.starts_with('{') {
        LogFormat::Json
    } else {
        LogFormat::Unknown
    }
}

fn parse_cef(log: &str, account_id: &String, host_id: &String) -> Result<NormalizedLog, ParseLogError> {
    let parts: Vec<&str> = log.split('|').collect();
    if parts.len() != 8 || !parts[0].starts_with("CEF:") {
        return Err(ParseLogError::InvalidCEFFormat);
    }

    let extension_part = parts[7];
    let mut extensions = HashMap::new();
    let mut current_pair = String::new();
    let mut in_quotes = false;

    for c in extension_part.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                current_pair.push(c);
            }
            ' ' if !in_quotes => {
                if !current_pair.is_empty() {
                    if let Some((key, value)) = current_pair.split_once('=') {
                        let value = value.trim_matches('"');
                        extensions.insert(key.to_string(), value.to_string());
                    }
                    current_pair.clear();
                }
            }
            _ => current_pair.push(c),
        }
    }

    if !current_pair.is_empty() {
        if let Some((key, value)) = current_pair.split_once('=') {
            let value = value.trim_matches('"');
            extensions.insert(key.to_string(), value.to_string());
        }
    }

    extensions.insert("version".to_string(), parts[0].replace("CEF:", ""));
    extensions.insert("device_vendor".to_string(), parts[1].to_string());
    extensions.insert("device_product".to_string(), parts[2].to_string());
    extensions.insert("device_version".to_string(), parts[3].to_string());
    extensions.insert("signature_id".to_string(), parts[4].to_string());
    extensions.insert("name".to_string(), parts[5].to_string());
    extensions.insert("severity".to_string(), parts[6].to_string());

    Ok(NormalizedLog {
        timestamp: extensions.get("rt").or(extensions.get("time")).map(String::from),
        src_ip: extensions.get("src").map(String::from),
        dst_ip: extensions.get("dst").map(String::from),
        event_type: Some(parts[5].to_string()),
        host_id: host_id.to_string(),
        account_id: account_id.to_string(),
        raw: log.to_string(),
        extensions,
    })
}

fn parse_syslog(log: &str, account_id: &String, host_id: &String) -> Result<NormalizedLog, ParseLogError> {
    let cleaned = clean_log(log);
    let mut parts = cleaned.split_whitespace();

    // Extract priority and timestamp
    let priority = parts.next().unwrap_or(""); // e.g., "<134>"
    let timestamp: String = parts.by_ref().take(3).collect::<Vec<&str>>().join(" ");
    let hostname = parts.next().unwrap_or("unknown");
    let message = parts.collect::<Vec<&str>>().join(" ");

    let mut extensions = HashMap::new();
    let mut src_ip = None;
    let mut dst_ip = None;
    let mut event_type = None;

    // Use regex to extract IPs and key details
    let ip_re = Regex::new(r"(?:SRC|from|client)\s*=?\s*(\d+\.\d+\.\d+\.\d+)").unwrap();
    let dst_re = Regex::new(r"DST=(\d+\.\d+\.\d+\.\d+)").unwrap();

    if let Some(cap) = ip_re.captures(&message) {
        src_ip = Some(cap[1].to_string());
        extensions.insert("src_ip".to_string(), cap[1].to_string());
    }
    if let Some(cap) = dst_re.captures(&message) {
        dst_ip = Some(cap[1].to_string());
        extensions.insert("dst_ip".to_string(), cap[1].to_string());
    }

    // Extract event type from common Syslog patterns
    if message.contains("sshd") {
        if message.contains("Failed password") {
            event_type = Some("failed_login".to_string());
        } else if message.contains("Accepted password") {
            event_type = Some("successful_login".to_string());
        }
    } else if message.contains("systemd") {
        event_type = Some("systemd_event".to_string());
    } else if message.contains("kernel") {
        event_type = Some("kernel_event".to_string());
    } else if message.contains("crond") {
        event_type = Some("cron_job".to_string());
    } else if message.contains("sudo") {
        event_type = Some("sudo_command".to_string());
    } else if message.contains("apache2") {
        event_type = Some("apache_error".to_string());
    }

    // Store additional metadata
    extensions.insert("priority".to_string(), priority.trim_matches(|c| c == '<' || c == '>').to_string());
    extensions.insert("hostname".to_string(), hostname.to_string());
    extensions.insert("message".to_string(), message.clone());

    Ok(NormalizedLog {
        timestamp: Some(timestamp),
        src_ip,
        dst_ip,
        event_type,
        host_id: host_id.to_string(),
        account_id: account_id.to_string(),
        raw: log.to_string(),
        extensions,
    })
}

fn parse_json(log: &str, account_id: &String, host_id: &String) -> Result<NormalizedLog, ParseLogError> {
    let fields: HashMap<String, Value> = serde_json::from_str(log)
        .map_err(|e| ParseLogError::InvalidFormat(format!("JSON parse error: {}", e)))?;
    let mut normalized = NormalizedLog {
        timestamp: None,
        src_ip: None,
        dst_ip: None,
        event_type: None,
        host_id: host_id.to_string(),
        account_id: account_id.to_string(),
        raw: log.to_string(),
        extensions: HashMap::new(),
    };

    for (key, value) in fields {
        // Convert Value to String, handling null and non-string types
        let value_str = match value {
            Value::String(s) => s,
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "".to_string(),
            _ => serde_json::to_string(&value).unwrap_or_default(),
        };

        match key.as_str() {
            "time" | "timestamp" => normalized.timestamp = Some(value_str),
            "src_ip" | "source_ip" | "src" => normalized.src_ip = Some(value_str.clone()),
            "dst_ip" | "dst" => normalized.dst_ip = Some(value_str.clone()),
            "event" | "event_type" | "message" => normalized.event_type = Some(value_str.clone()),
            _ => { normalized.extensions.insert(key, value_str); }
        }
    }
    Ok(normalized)
}

pub fn process_log(
    log: &str,
    account_id: &String,
    host_id: &String,
) -> Result<(String, String), ParseLogError> {
    let cleaned = clean_log(log);
    let format = detect_format(&cleaned);

    let normalized = match format {
        LogFormat::Cef => parse_cef(&cleaned, account_id, host_id)?,
        LogFormat::Syslog => parse_syslog(&cleaned, account_id, host_id)?,
        LogFormat::Json => parse_json(&cleaned, account_id, host_id)?,
        LogFormat::Unknown => {
            return Err(ParseLogError::InvalidFormat("Unknown log format".to_string()));
        }
    };

    let log_json = serde_json::to_string(&normalized)
        .map_err(|e| ParseLogError::SerializationError(format!("Serialization error: {}", e)))?;
    let timestamp = normalized.timestamp.unwrap_or_default();

    Ok((log_json, timestamp))
}