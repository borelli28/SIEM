use std::fs::File;
use std::io::{self, BufReader, BufRead};
use crate::global::GLOBAL_MESSAGE_QUEUE;

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

    for line in reader.lines() {
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
    Ok(())
}