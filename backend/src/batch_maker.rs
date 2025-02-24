use std::io::{self, BufReader, BufRead};
use crate::global::GLOBAL_MESSAGE_QUEUE;
use std::fs::File;
use serde_json;

// A Batch is <= 50 log entries long
#[derive(Clone)]
pub struct Batch {
    pub lines: Vec<String>,
}

impl Batch {
    pub fn new() -> Self {
        Batch {
            lines: Vec::new(),
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
    }

    pub fn is_full(&self) -> bool {
        self.lines.len() >= 50
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

pub async fn create_batches(file_path: &str) -> Result<(), io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut current_batch = Batch::new();

    // Read the first line to determine format
    let mut lines = reader.lines();
    if let Some(first_line) = lines.next() {
        let first_line = first_line?;
        if first_line.trim().starts_with('[') {
            // JSON array detected
            let mut json_content = first_line;
            for line in lines {
                json_content.push_str("\n");
                json_content.push_str(&line?);
            }

            // Parse JSON array into individual objects
            let log_entries: Vec<serde_json::Value> = serde_json::from_str(&json_content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse JSON array: {}", e)))?;

            for entry in log_entries {
                let entry_str = serde_json::to_string(&entry)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to serialize JSON entry: {}", e)))?;
                current_batch.add_line(entry_str);

                if current_batch.is_full() {
                    let queue = GLOBAL_MESSAGE_QUEUE.lock().await;
                    queue.enqueue(current_batch.clone()).await.map_err(|e| {
                        eprintln!("Error enqueuing batch: {}", e);
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;
                    current_batch.clear();
                }
            }
        } else {
            // Non-JSON, process line-by-line
            current_batch.add_line(first_line);
            if current_batch.is_full() {
                let queue = GLOBAL_MESSAGE_QUEUE.lock().await;
                queue.enqueue(current_batch.clone()).await.map_err(|e| {
                    eprintln!("Error enqueuing batch: {}", e);
                    io::Error::new(io::ErrorKind::Other, e)
                })?;
                current_batch.clear();
            }

            for line in lines {
                let line = line?;
                current_batch.add_line(line);

                if current_batch.is_full() {
                    let queue = GLOBAL_MESSAGE_QUEUE.lock().await;
                    queue.enqueue(current_batch.clone()).await.map_err(|e| {
                        eprintln!("Error enqueuing batch: {}", e);
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;
                    current_batch.clear();
                }
            }
        }
    }

    // Enqueue any remaining lines
    if !current_batch.lines.is_empty() {
        let queue = GLOBAL_MESSAGE_QUEUE.lock().await;
        queue.enqueue(current_batch.clone()).await.map_err(|e| {
            eprintln!("Error enqueuing batch: {}", e);
            io::Error::new(io::ErrorKind::Other, e)
        })?;
    }

    Ok(())
}