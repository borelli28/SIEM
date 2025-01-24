use std::sync::{Mutex, atomic::{AtomicU16, Ordering}};
use crate::rules::evaluate_log_against_rules;
use crate::global::GLOBAL_MESSAGE_QUEUE;
use serde::{Deserialize, Serialize};
use crate::log::{Log, create_log};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub account_id: String,
    pub host_id: String,
    pub line_number: u16,
    pub version: String,
    pub device_vendor: String,
    pub device_product: String,
    pub device_version: String,
    pub signature_id: String,
    pub name: String,
    pub severity: String,
    pub extensions: HashMap<String, String>,
}

pub struct LogCollector {
    logs: Mutex<Vec<LogEntry>>,
    next_id: AtomicU16,
}

impl LogCollector {
    pub fn new() -> Self {
        Self {
            logs: Mutex::new(Vec::new()),
            next_id: AtomicU16::new(1),
        }
    }

    pub fn add_log(&self, mut log: LogEntry) -> u16 {
        let line_number = self.next_id.fetch_add(1, Ordering::SeqCst);
        log.line_number = line_number;
        let mut logs = self.logs.lock().unwrap();
        logs.push(log);
        line_number
    }
}

#[derive(Debug)]
pub enum ParseLogError {
    InvalidCEFFormat,
    BatchDequeueError,
    DatabaseError(String),
    AlertEvaluationError(String)

}

impl std::fmt::Display for ParseLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseLogError::InvalidCEFFormat => write!(f, "Invalid CEF format"),
            ParseLogError::BatchDequeueError => write!(f, "Error while retriveing batch"),
            ParseLogError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ParseLogError::AlertEvaluationError(msg) => write!(f, "Alert evaluation error: {}", msg),
        }
    }
}

impl std::error::Error for ParseLogError {}

/// Parses a CEF (Common Event Format) log string into a LogEntry struct
/// Example CEF format:
/// CEF:0|Borelli28|Blog|1.0|3007|Authentication failed|Medium|rt=2024-12-20T00:23:57.409-05:00 level=info msg="No token provided"
pub fn log_parser(cef_str: &str, account_id: &String, host_id_param: &String) 
-> Result<LogEntry, ParseLogError> {
    // Split the CEF string by pipe character into its main components
    // CEF format has 8 parts: CEF:Version|Vendor|Product|Version|SignatureID|Name|Severity|Extensions
    let parts: Vec<&str> = cef_str.split('|').collect();

    // Validate basic CEF format requirements
    if parts.len() != 8 || !parts[0].starts_with("CEF:") {
        return Err(ParseLogError::InvalidCEFFormat);
    }

    // Get the extensions part (last part after the 7th pipe)
    // Contains key-value pairs like: rt=timestamp level=info msg="value with spaces"
    let extension_part = parts[7];
    let mut extensions = HashMap::new();
    let mut current_pair = String::new();

    // Track whether we're inside quoted values
    // This allows handling values that contain spaces
    let mut in_quotes = false;

    // Process each character in the extensions part
    for c in extension_part.chars() {
        match c {
            // Handle quotes - toggle quote state and include quote in pair
            '"' => {
                in_quotes = !in_quotes;  // Toggle between in/out of quotes
                current_pair.push(c);    // Keep quotes in the string for later trimming
            }
            // Handle spaces - only split pairs on spaces outside quotes
            ' ' if !in_quotes => {
                // Process the completed key=value pair
                if !current_pair.is_empty() {
                    if let Some((key, value)) = current_pair.split_once('=') {
                        // Remove surrounding quotes from value if present
                        let value = value.trim_matches('"');
                        extensions.insert(key.to_string(), value.to_string());
                    }
                    current_pair.clear();  // Reset for next pair
                }
            }
            // Collect all other characters into the current pair
            _ => current_pair.push(c),
        }
    }

    // Process the final key-value pair if one exists
    // Needed because there's no trailing space after the last pair
    if !current_pair.is_empty() {
        if let Some((key, value)) = current_pair.split_once('=') {
            let value = value.trim_matches('"');
            extensions.insert(key.to_string(), value.to_string());
        }
    }

    Ok(LogEntry {
        host_id: host_id_param.to_string(),
        account_id: account_id.to_string(),
        line_number: 0,
        version: parts[0].replace("CEF:", ""),     // Remove "CEF:" prefix from version
        device_vendor: parts[1].to_string(),
        device_product: parts[2].to_string(),
        device_version: parts[3].to_string(),
        signature_id: parts[4].to_string(),
        name: parts[5].to_string(),
        severity: parts[6].to_string(),
        extensions,
    })
}

pub async fn process_logs(collector: &LogCollector, account_id: String, host_id: String) -> Result<(), ParseLogError> {
    let queue = GLOBAL_MESSAGE_QUEUE.lock().await;
    let batch = queue.dequeue().await.map_err(|e| {
        eprintln!("Error dequeuing batch: {}", e);
        ParseLogError::BatchDequeueError
    })?;

    for cef_log in batch.lines {
        let log_entry = log_parser(&cef_log, &account_id, &host_id)?;
        collector.add_log(log_entry.clone());

        // Insert Log into DB
        let extensions_json = serde_json::to_string(&log_entry.extensions)
            .map_err(|e| ParseLogError::DatabaseError(format!("Failed to serialize extensions: {}", e)))?;
        let new_log = Log {
            id: String::new(),
            hash: String::from("Temp hash"),
            account_id: log_entry.account_id.clone(),
            host_id: log_entry.host_id.clone(),
            version: Some(log_entry.version.clone()),
            device_vendor: Some(log_entry.device_vendor.clone()),
            device_product: Some(log_entry.device_product.clone()),
            device_version: Some(log_entry.device_version.clone()),
            signature_id: Some(log_entry.signature_id.clone()),
            name: Some(log_entry.name.clone()),
            severity: Some(log_entry.severity.clone()),
            extensions: Some(extensions_json),
        };

        if let Err(e) = create_log(&new_log) {
            eprintln!("Error inserting log into database: {}", e);
            return Err(ParseLogError::DatabaseError(format!(
                "Error inserting log into database. Line number: {}",
                log_entry.line_number
            )));
        }

        // After inserting log, check it against alert rules
        match evaluate_log_against_rules(&log_entry, &account_id).await {
            Ok(alerts) => alerts,
            Err(err) => {
                eprintln!("Error evaluating alerts: {:?}", err);
                return Err(ParseLogError::AlertEvaluationError(err.to_string()));
            }
        };
    }

    Ok(())
}