use crate::message_queue::MessageQueue;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use std::sync::{Arc};

// Single global message queue instance
lazy_static! {
    pub static ref GLOBAL_MESSAGE_QUEUE: Arc<Mutex<MessageQueue>> = Arc::new(Mutex::new(MessageQueue::new()));
}