use crate::log_parser::{process_log, ParseLogError, NormalizedLog};
use std::sync::{Mutex, atomic::{AtomicU16, Ordering}};
use crate::rules::evaluate_log_against_rules;
use crate::global::GLOBAL_MESSAGE_QUEUE;
use crate::log::{Log, create_log};
use serde_json;

pub struct LogCollector {
    logs: Mutex<Vec<Log>>,
    next_id: AtomicU16,
}

impl LogCollector {
    pub fn new() -> Self {
        Self {
            logs: Mutex::new(Vec::new()),
            next_id: AtomicU16::new(1),
        }
    }

    pub fn add_log(&self, log: Log) -> u16 {
        let line_number = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut logs = self.logs.lock().unwrap();
        logs.push(log);
        line_number
    }
}

pub async fn process_logs(collector: &LogCollector, account_id: String, host_id: String) -> Result<(), ParseLogError> {
    let queue = GLOBAL_MESSAGE_QUEUE.lock().await;
    let batch = match queue.dequeue().await {
        Ok(batch) => batch,
        Err(_) => return Ok(()), // Queue empty
    };

    if batch.lines.is_empty() {
        return Ok(());
    }

    for cef_log in batch.lines {
        let id = format!("log{}", collector.next_id.fetch_add(1, Ordering::SeqCst));
        let hash = "temp_hash".to_string();

        // Parse the log
        let (log_json, timestamp) = process_log(&cef_log, &account_id, &host_id)?;

        let log = Log {
            id,
            hash: hash.clone(),
            account_id: account_id.clone(),
            host_id: host_id.clone(),
            timestamp: Some(timestamp),
            log_data: log_json.clone(),
        };

        // Store the log
        match create_log(&log) {
            Ok(Some(new_log)) => {
                collector.add_log(new_log.clone());

                let normalized_log: NormalizedLog = serde_json::from_str(&log_json)
                    .map_err(|e| ParseLogError::SerializationError(format!("Failed to deserialize log for rule evaluation: {}", e)))?;
                match evaluate_log_against_rules(&normalized_log, &account_id).await {
                    Ok(_alerts) => (),
                    Err(err) => return Err(ParseLogError::DatabaseError(format!("Rule evaluation error: {}", err))),
                }
            }
            Ok(None) => println!("Duplicate log skipped"),
            Err(e) => return Err(ParseLogError::DatabaseError(e.to_string())),
        }
    }

    Ok(())
}