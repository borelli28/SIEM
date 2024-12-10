use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    version: String,
    device_vendor: String,
    device_product: String,
    device_version: String,
    signature_id: String,
    name: String,
    severity: String,
    extensions: HashMap<String, String>,
}

pub struct LogCollector {
    logs: Mutex<Vec<LogEntry>>,
}

impl LogCollector {
    pub fn new() -> Self {
        Self {
            logs: Mutex::new(Vec::new()),
        }
    }

    pub fn add_log(&self, log: LogEntry) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(log);
    }

    pub fn get_logs(&self) -> Vec<LogEntry> {
        let logs = self.logs.lock().unwrap();
        logs.clone()
    }
}

#[derive(Debug)]
pub enum ParseLogError {
    InvalidCEFFormat,
}

impl std::fmt::Display for ParseLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseLogError::InvalidCEFFormat => write!(f, "Invalid CEF format"),
        }
    }
}

impl std::error::Error for ParseLogError {}

pub fn parse_cef_log(cef_str: &str) -> Result<LogEntry, ParseLogError> {
    let parts: Vec<&str> = cef_str.split('|').collect();
    // Validate CEF format
    if parts.len() != 8 || !parts[0].starts_with("CEF:") {
        return Err(ParseLogError::InvalidCEFFormat);
    }
    let extension_part = parts[7];

    let mut extensions = HashMap::new();
    for pair in extension_part.split_whitespace() {
        // Split each pair into key and value
        let kv: Vec<&str> = pair.splitn(2, '=').collect();
        // If we have both a key & value
        if kv.len() == 2 {
            // Insert the key-value pair into the extensions HashMap
            // We trim quotes from the value and convert both key and value to owned Strings
            extensions.insert(kv[0].to_string(), kv[1].trim_matches('"').to_string());
        }
    }

    Ok(LogEntry {
        version: parts[0].replace("CEF:", ""),
        device_vendor: parts[1].to_string(),
        device_product: parts[2].to_string(),
        device_version: parts[3].to_string(),
        signature_id: parts[4].to_string(),
        name: parts[5].to_string(),
        severity: parts[6].to_string(),
        extensions,
    })
}