use std::fs::File;
use std::io::{self, BufReader, BufRead};
use crate::message_queue::MessageQueue;

// A Batch is <= 1000 log entries long
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
        self.lines.len() >= 1000
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

pub fn create_batch(file_path: &str, message_queue: &MessageQueue) -> Result<(), io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut current_batch = Batch::new();

    for line in reader.lines() {
        let line = line?;
        current_batch.add_line(line);

        if current_batch.is_full() {
            message_queue.enqueue(current_batch.clone()).map_err(|e| {
                eprintln!("Error enqueuing batch: {}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })?;
            current_batch.clear();
        }
    }
    Ok(())
}