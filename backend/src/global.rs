use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::message_queue::MessageQueue;

// Single global message queue instance
lazy_static! {
    pub static ref GLOBAL_MESSAGE_QUEUE: Arc<Mutex<MessageQueue>> = Arc::new(Mutex::new(MessageQueue::new()));
}