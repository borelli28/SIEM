use crate::rules::evaluate_log_against_rules;
use crate::log::{Log, create_log};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, atomic::{AtomicU16, Ordering}};
use std::collections::HashMap;
use crate::global::GLOBAL_MESSAGE_QUEUE;

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

    pub fn get_logs(&self) -> Vec<LogEntry> {
        let logs = self.logs.lock().unwrap();
        logs.clone()
    }

    pub fn get_last_processed_id(&self) -> u16 {
        self.next_id.load(Ordering::SeqCst) - 1
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
            ParseLogError::DatabaseError(_) => write!(f, "Error while inserting in database"),
            ParseLogError::AlertEvaluationError(_) => write!(f, "Error while matching log with alert rules"),
        }
    }
}

impl std::error::Error for ParseLogError {}

pub fn parse_cef_log(cef_str: &str, account_id: &String, host_id_param: &String) -> Result<LogEntry, ParseLogError> {
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
        host_id: host_id_param.to_string(),
        account_id: account_id.to_string(),
        line_number: 0,
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

pub async fn process_logs(collector: &LogCollector, account_id: String, host_id: String) -> Result<(), ParseLogError> {
    let queue = GLOBAL_MESSAGE_QUEUE.lock().await;
    let batch = queue.dequeue().await.map_err(|e| {
        eprintln!("Error dequeuing batch: {}", e);
        ParseLogError::BatchDequeueError
    })?;

    for cef_log in batch.lines {
        let log_entry = parse_cef_log(&cef_log, &account_id, &host_id)?;
        collector.add_log(log_entry.clone());

        // Insert Log into DB
        let extensions_json = serde_json::to_string(&log_entry.extensions)
            .map_err(|e| ParseLogError::DatabaseError(format!("Failed to serialize extensions: {}", e)))?;
        let new_log = Log {
            id: String::new(),
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