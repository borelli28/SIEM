use std::collections::VecDeque;
use crate::batch_maker::Batch;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct MessageQueue {
    queue: Arc<Mutex<VecDeque<Batch>>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn enqueue(&self, batch: Batch) -> Result<(), String> {
        let mut queue = self.queue.lock().await;
        queue.push_back(batch);
        Ok(())
    }

    pub async fn dequeue(&self) -> Result<Batch, String> {
        let mut queue = self.queue.lock().await;
        queue.pop_front().ok_or("Queue is empty".to_string())
    }

    pub async fn is_empty(&self) -> bool {
        let queue = self.queue.lock().await;
        queue.is_empty()
    }
}