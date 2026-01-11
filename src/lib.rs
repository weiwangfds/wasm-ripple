#![allow(clippy::await_holding_refcell_ref)]
#![allow(dead_code)]

mod constants;
mod ring_buffer;
mod types;
mod js_utils;
mod utils;
mod inner_queue;
mod queue;

// Re-export the main MessageQueue type and its dependencies
pub use queue::MessageQueue;
pub use types::{Message, Topic, InnerQueue};



// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::ERR_WINDOW_NOT_AVAILABLE;
    use crate::constants::ERR_CRYPTO_NOT_AVAILABLE;
    use crate::types::Topic;

    #[test]
    fn test_message_creation() {
        // Skip this test on non-WASM targets since JsValue requires WASM
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Test skipped on non-WASM targets
        }

        #[cfg(target_arch = "wasm32")]
        {
            let msg = Message {
                id: 12345,
                topic_id: 1,
                payload: JsValue::from_str("test payload"),
                timestamp: 12345.0,
                origin_id: Rc::new("test-origin".to_string()),
            };

            assert_eq!(msg.id, 12345);
            assert_eq!(msg.topic_id, 1);
            assert_eq!(msg.timestamp, 12345.0);
            assert_eq!(*msg.origin_id, "test-origin");
        }
    }

    #[test]
    fn test_topic_new() {
        let topic = Topic::new("test".to_string());
        assert_eq!(topic.next_id, 0);
        assert!(topic.subscribers.is_empty());
        assert_eq!(topic.name, "test");
    }

    #[test]
    fn test_constants() {
        assert_eq!(ERR_WINDOW_NOT_AVAILABLE, "Window not available");
        assert_eq!(ERR_CRYPTO_NOT_AVAILABLE, "Crypto not available");
    }

    #[test]
    fn test_inner_queue_default() {
        use std::collections::HashMap;
        use std::rc::Rc;
        let queue = InnerQueue {
            topics: Vec::new(),
            topic_index: HashMap::new(),
            channel: None,
            client_id: Rc::new("test-client".to_string()),
        };

        assert!(queue.topics.is_empty());
        assert!(queue.topic_index.is_empty());
        assert!(queue.channel.is_none());
        assert_eq!(*queue.client_id, "test-client");
    }
}
