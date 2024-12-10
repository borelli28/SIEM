use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crate::batch_maker::{Batch};

#[derive(Clone)]
pub struct MessageQueue {
    queue: Arc<Mutex<VecDeque<Batch>>>, // Thread-safe queue
}

impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn enqueue(&self, batch: Batch) -> Result<(), String> {
        match self.queue.lock() {
            Ok(mut queue) => {
                queue.push_back(batch);
                Ok(())
            }
            Err(_) => Err("Failed to lock the queue".to_string()),
        }
    }

    pub fn dequeue(&self) -> Result<Batch, String> {
        match self.queue.lock() {
            Ok(mut queue) => {
                queue.pop_front().ok_or("Queue is empty".to_string())
            }
            Err(_) => Err("Failed to lock the queue".to_string()),
        }
    }

    // Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        let queue = self.queue.lock().unwrap();
        queue.is_empty()
    }
}