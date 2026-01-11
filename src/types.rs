use crate::ring_buffer::RingBuffer;
use std::collections::HashMap;
use web_sys::BroadcastChannel;
use wasm_bindgen::JsValue;
use js_sys::Function;
use std::rc::Rc;

/// A lightweight message struct for internal message queue logic.
/// The payload is handled as raw JsValue to avoid serialization overhead.
#[derive(Clone)]
pub struct Message {
    /// Unique message identifier
    /// Uses u64 to avoid heap allocation for small messages
    /// Only converted to string when crossing JS boundary
    pub id: u64,
    /// Topic ID this message belongs to (optimized from String)
    pub topic_id: u32,
    /// Message payload (any JavaScript value)
    pub payload: JsValue,
    /// Timestamp when the message was created (milliseconds since epoch)
    pub timestamp: f64,
    /// ID of the client that originated this message
    pub origin_id: Rc<String>,
}

/// Represents a topic with its subscribers
pub struct Topic {
    /// The name of the topic
    pub name: String,
    /// Map of subscriber ID to callback function
    pub subscribers: HashMap<u32, Function>,
    /// Next subscriber ID to assign
    pub next_id: u32,
    /// Optional message buffer (ring buffer) for caching messages
    /// If None, messages are not buffered
    buffer: Option<RingBuffer>,
}

impl Topic {
    pub fn new(name: String) -> Self {
        Topic {
            name,
            subscribers: HashMap::new(),
            next_id: 0,
            buffer: None,
        }
    }

    /// Create a topic with a message buffer
    pub fn with_buffer(name: String, capacity: usize) -> Self {
        Topic {
            name,
            subscribers: HashMap::new(),
            next_id: 0,
            buffer: Some(RingBuffer::new(capacity)),
        }
    }

    /// Enable message buffering with the given capacity
    pub fn enable_buffer(&mut self, capacity: usize) -> Option<RingBuffer> {
        self.buffer.replace(RingBuffer::new(capacity))
    }

    /// Disable message buffering
    pub fn disable_buffer(&mut self) -> Option<RingBuffer> {
        self.buffer.take()
    }

    /// Check if this topic has buffering enabled
    pub fn has_buffer(&self) -> bool {
        self.buffer.is_some()
    }

    /// Get the buffer if it exists
    pub fn get_buffer(&self) -> Option<&RingBuffer> {
        self.buffer.as_ref()
    }

    /// Get mutable reference to the buffer if it exists
    pub fn get_buffer_mut(&mut self) -> Option<&mut RingBuffer> {
        self.buffer.as_mut()
    }
}

/// Internal queue state
#[derive(Default)]
pub struct InnerQueue {
    /// List of topics, accessible by index (ID)
    pub topics: Vec<Topic>,
    /// Map of topic name to topic index (ID)
    pub topic_index: HashMap<String, usize>,
    /// Optional broadcast channel for cross-tab communication
    pub channel: Option<BroadcastChannel>,
    /// Unique client identifier
    pub client_id: Rc<String>,
    /// Set of seen message IDs to prevent duplicates (especially during sync)
    pub seen_ids: std::collections::HashSet<u64>,
}

impl InnerQueue {
    pub fn get_topic(&self, name: &str) -> Option<&Topic> {
        self.topic_index.get(name).map(|&idx| &self.topics[idx])
    }

    pub fn get_topic_mut(&mut self, name: &str) -> Option<&mut Topic> {
        self.topic_index.get(name).map(|&idx| &mut self.topics[idx])
    }
    
    pub fn get_topic_by_id(&self, id: usize) -> Option<&Topic> {
        self.topics.get(id)
    }

    pub fn get_topic_by_id_mut(&mut self, id: usize) -> Option<&mut Topic> {
        self.topics.get_mut(id)
    }

    pub fn get_or_create_topic_id(&mut self, name: &str) -> usize {
        if let Some(&id) = self.topic_index.get(name) {
            id
        } else {
            let id = self.topics.len();
            self.topics.push(Topic::new(name.to_string()));
            self.topic_index.insert(name.to_string(), id);
            id
        }
    }
}
