use std::rc::Rc;
use crate::types::Message;

/// A fixed-size circular buffer for storing messages
/// Provides O(1) push and pop operations without memory allocation
#[derive(Clone)]
pub struct RingBuffer {
    /// The underlying buffer
    buffer: Vec<Option<Rc<Message>>>,
    /// Maximum capacity
    capacity: usize,
    /// Current number of elements
    size: usize,
    /// Index of the front element
    front: usize,
    /// Index where the next element will be inserted
    rear: usize,
}

impl RingBuffer {
    /// Create a new ring buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        RingBuffer {
            buffer: vec![None; capacity],
            capacity,
            size: 0,
            front: 0,
            rear: 0,
        }
    }

    /// Get the current number of messages in the buffer
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    /// Get the maximum capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Push a message into the buffer
    /// Returns the oldest message if the buffer was full (overwrites)
    pub fn push(&mut self, msg: Rc<Message>) -> Option<Rc<Message>> {
        if self.capacity == 0 {
            return Some(msg); // Reject if capacity is 0
        }

        let displaced = if self.is_full() {
            // Buffer is full, remove and return the front element
            let old = self.buffer[self.front].take();
            self.front = (self.front + 1) % self.capacity;
            old
        } else {
            None
        };

        // Insert the new message
        self.buffer[self.rear] = Some(msg);
        self.rear = (self.rear + 1) % self.capacity;

        if !displaced.is_some() {
            self.size += 1;
        }

        displaced
    }

    /// Pop the oldest message from the buffer
    pub fn pop(&mut self) -> Option<Rc<Message>> {
        if self.is_empty() {
            return None;
        }

        let msg = self.buffer[self.front].take();
        self.front = (self.front + 1) % self.capacity;
        self.size -= 1;

        msg
    }

    /// Peek at the oldest message without removing it
    pub fn peek(&self) -> Option<&Rc<Message>> {
        if self.is_empty() {
            return None;
        }
        self.buffer[self.front].as_ref()
    }

    /// Peek at the newest message without removing it
    pub fn peek_back(&self) -> Option<&Rc<Message>> {
        if self.is_empty() {
            return None;
        }
        let idx = if self.rear == 0 {
            self.capacity - 1
        } else {
            self.rear - 1
        };
        self.buffer[idx].as_ref()
    }

    /// Clear all messages from the buffer
    pub fn clear(&mut self) {
        for item in self.buffer.iter_mut() {
            *item = None;
        }
        self.size = 0;
        self.front = 0;
        self.rear = 0;
    }

    /// Get all messages as a vector (oldest first)
    pub fn to_vec(&self) -> Vec<Rc<Message>> {
        let mut result = Vec::with_capacity(self.size);
        let mut idx = self.front;

        for _ in 0..self.size {
            if let Some(msg) = &self.buffer[idx] {
                result.push(msg.clone());
            }
            idx = (idx + 1) % self.capacity;
        }

        result
    }

    /// Iterate over all messages from oldest to newest
    pub fn iter(&self) -> RingBufferIter<'_> {
        RingBufferIter {
            buffer: self,
            index: 0,
            count: 0,
        }
    }
}

/// Iterator over ring buffer messages
pub struct RingBufferIter<'a> {
    buffer: &'a RingBuffer,
    index: usize,
    count: usize,
}

impl<'a> Iterator for RingBufferIter<'a> {
    type Item = &'a Rc<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.buffer.size {
            return None;
        }

        let idx = (self.buffer.front + self.count) % self.buffer.capacity;
        self.count += 1;

        self.buffer.buffer[idx].as_ref()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.size - self.count;
        (remaining, Some(remaining))
    }
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self::new(100) // Default capacity of 100 messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_new() {
        let rb = RingBuffer::new(10);
        assert_eq!(rb.capacity(), 10);
        assert_eq!(rb.len(), 0);
        assert!(rb.is_empty());
        assert!(!rb.is_full());
    }

    #[test]
    fn test_ring_buffer_push_pop() {
        // Skip this test on non-WASM targets since JsValue requires WASM
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Test skipped on non-WASM targets
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;

            let mut rb = RingBuffer::new(3);

            let msg1 = Message {
                id: "1".to_string(),
                topic: "test".to_string(),
                payload: JsValue::from_str("data1"),
                timestamp: 1.0,
                origin_id: Rc::new("client1".to_string()),
            };

            let msg2 = Message {
                id: "2".to_string(),
                topic: "test".to_string(),
                payload: JsValue::from_str("data2"),
                timestamp: 2.0,
                origin_id: Rc::new("client1".to_string()),
            };

            // Push two messages
            assert!(rb.push(Rc::new(msg1.clone())).is_none());
            assert!(rb.push(Rc::new(msg2.clone())).is_none());

            assert_eq!(rb.len(), 2);

            // Pop one message
            let popped = rb.pop().unwrap();
            assert_eq!(popped.id, "1");
            assert_eq!(rb.len(), 1);

            // Peek at the next message
            let peeked = rb.peek().unwrap();
            assert_eq!(peeked.id, "2");
        }
    }

    #[test]
    fn test_ring_buffer_overflow() {
        // Skip this test on non-WASM targets since JsValue requires WASM
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Test skipped on non-WASM targets
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;

            let mut rb = RingBuffer::new(2);

            let msg1 = Message {
                id: "1".to_string(),
                topic: "test".to_string(),
                payload: JsValue::UNDEFINED,
                timestamp: 1.0,
                origin_id: Rc::new("client1".to_string()),
            };

            let msg2 = Message {
                id: "2".to_string(),
                topic: "test".to_string(),
                payload: JsValue::UNDEFINED,
                timestamp: 2.0,
                origin_id: Rc::new("client1".to_string()),
            };

            let msg3 = Message {
                id: "3".to_string(),
                topic: "test".to_string(),
                payload: JsValue::UNDEFINED,
                timestamp: 3.0,
                origin_id: Rc::new("client1".to_string()),
            };

            rb.push(Rc::new(msg1));
            rb.push(Rc::new(msg2));

            // This should overflow and return msg1
            let displaced = rb.push(Rc::new(msg3));
            assert!(displaced.is_some());
            assert_eq!(displaced.unwrap().id, "1");

            assert_eq!(rb.len(), 2);
            assert_eq!(rb.peek().unwrap().id, "2");
            assert_eq!(rb.peek_back().unwrap().id, "3");
        }
    }

    #[test]
    fn test_ring_buffer_peek() {
        // Skip this test on non-WASM targets since JsValue requires WASM
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Test skipped on non-WASM targets
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;

            let mut rb = RingBuffer::new(3);

            let msg1 = Message {
                id: "1".to_string(),
                topic: "test".to_string(),
                payload: JsValue::UNDEFINED,
                timestamp: 1.0,
                origin_id: "client1".to_string(),
            };

            let msg2 = Message {
                id: "2".to_string(),
                topic: "test".to_string(),
                payload: JsValue::UNDEFINED,
                timestamp: 2.0,
                origin_id: "client1".to_string(),
            };

            rb.push(msg1);
            rb.push(msg2);

            // Peek front (oldest)
            let front = rb.peek().unwrap();
            assert_eq!(front.id, "1");

            // Peek back (newest)
            let back = rb.peek_back().unwrap();
            assert_eq!(back.id, "2");
        }
    }

    #[test]
    fn test_ring_buffer_clear() {
        // Skip this test on non-WASM targets since JsValue requires WASM
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Test skipped on non-WASM targets
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;

            let mut rb = RingBuffer::new(3);

            let msg = Message {
                id: "1".to_string(),
                topic: "test".to_string(),
                payload: JsValue::UNDEFINED,
                timestamp: 1.0,
                origin_id: "client1".to_string(),
            };

            rb.push(msg);
            assert_eq!(rb.len(), 1);

            rb.clear();
            assert_eq!(rb.len(), 0);
            assert!(rb.is_empty());
        }
    }

    #[test]
    fn test_ring_buffer_to_vec() {
        // Skip this test on non-WASM targets since JsValue requires WASM
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Test skipped on non-WASM targets
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsValue;

            let mut rb = RingBuffer::new(3);

            for i in 1..=3 {
                rb.push(Message {
                    id: i.to_string(),
                    topic: "test".to_string(),
                    payload: JsValue::UNDEFINED,
                    timestamp: i as f64,
                    origin_id: "client1".to_string(),
                });
            }

            let vec = rb.to_vec();
            assert_eq!(vec.len(), 3);
            assert_eq!(vec[0].id, "1");
            assert_eq!(vec[1].id, "2");
            assert_eq!(vec[2].id, "3");
        }
    }
}
