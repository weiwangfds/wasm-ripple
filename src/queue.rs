use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use web_sys::{BroadcastChannel, MessageEvent};
use js_sys::{Promise, Function, Array};

use crate::types::{InnerQueue, Message};
use crate::utils::generate_uuid;
use crate::js_utils::parse_js_message;

/// A WebAssembly-based message queue with support for:
/// - Topic-based pub/sub messaging
/// - Cross-tab communication via BroadcastChannel
/// - Synchronous and asynchronous publishing
/// - Zero-copy message passing (using JsValue)
#[wasm_bindgen]
pub struct MessageQueue {
    /// Internal queue state (wrapped in `Rc<RefCell>` for shared mutable access)
    inner: Rc<RefCell<InnerQueue>>,
    /// Closure for broadcast channel event handler
    _closure: Option<Closure<dyn FnMut(MessageEvent)>>,
}

#[wasm_bindgen]
impl MessageQueue {
    #[wasm_bindgen(constructor)]
    pub fn new(channel_name: Option<String>) -> Result<MessageQueue, JsValue> {
        let client_id = generate_uuid()?;

        let channel = if let Some(name) = channel_name {
            Some(BroadcastChannel::new(&name).map_err(|_| {
                JsValue::from_str("Failed to create BroadcastChannel")
            })?)
        } else {
            None
        };

        let inner = Rc::new(RefCell::new(InnerQueue {
            topics: Vec::new(),
            topic_index: HashMap::new(),
            channel: channel.clone(),
            client_id: Rc::new(client_id.clone()),
            seen_ids: std::collections::HashSet::new(),
        }));

        // Setup BroadcastChannel listener if it exists
        let inner_clone = inner.clone();
        let closure = if channel.is_some() {
            let cb = Closure::wrap(Box::new(move |event: MessageEvent| {
                let data = event.data();
                let mut queue = inner_clone.borrow_mut();

                // Helper to process a message
                let mut process_msg = |queue: &mut InnerQueue, msg_val: JsValue| {
                    if let Ok((mut m, topic_name)) = parse_js_message(&msg_val) {
                        if !queue.seen_ids.contains(&m.id) {
                            queue.seen_ids.insert(m.id);
                            
                            // Resolve topic ID
                            let topic_id = queue.get_or_create_topic_id(&topic_name) as u32;
                            m.topic_id = topic_id;
                            
                            if *m.origin_id != *queue.client_id {
                                 queue.dispatch_local(&Rc::new(m), None);
                            }
                        }
                    }
                };

                if data.is_array() {
                    let arr = Array::from(&data);
                    if let Some(type_val) = arr.get(0).as_f64() {
                        match type_val as u8 {
                            0 => { // PUB: [0, msg]
                                process_msg(&mut queue, arr.get(1));
                            },
                            1 => { // SYNC_REQ: [1, origin_id]
                                let origin_id = arr.get(1).as_string().unwrap_or_default();
                                if origin_id != *queue.client_id {
                                    // Send all buffered messages
                                    let all_msgs = Array::new();
                                    for topic in &queue.topics {
                                        if let Some(buffer) = topic.get_buffer() {
                                            for msg in buffer.iter() {
                                                if let Ok(msg_js) = crate::js_utils::message_to_js(msg, &topic.name) {
                                                    all_msgs.push(&msg_js);
                                                }
                                            }
                                        }
                                    }
                                    
                                    if all_msgs.length() > 0 {
                                        if let Some(ref ch) = queue.channel {
                                            let resp = Array::new();
                                            resp.push(&JsValue::from(2)); // SYNC_RESP
                                            resp.push(&all_msgs);
                                            let _ = ch.post_message(&resp);
                                        }
                                    }
                                }
                            },
                            2 => { // SYNC_RESP: [2, [msg1, msg2...]]
                                let msgs = Array::from(&arr.get(1));
                                for i in 0..msgs.length() {
                                    process_msg(&mut queue, msgs.get(i));
                                }
                            },
                            _ => {}
                        }
                    }
                } else if data.is_object() {
                    // Fallback for backward compatibility
                    process_msg(&mut queue, data);
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            if let Some(ref c) = inner.borrow().channel {
                c.set_onmessage(Some(cb.as_ref().unchecked_ref()));
                
                // Send SYNC_REQ
                // [1, client_id]
                let req = Array::new();
                req.push(&JsValue::from(1));
                req.push(&JsValue::from(inner.borrow().client_id.as_str()));
                let _ = c.post_message(&req);
            }
            Some(cb)
        } else {
            None
        };

        Ok(MessageQueue {
            inner,
            _closure: closure,
        })
    }

    pub fn create_topic(&self, topic_name: &str) -> bool {
        let mut queue = self.inner.borrow_mut();
        if queue.topic_index.contains_key(topic_name) {
            false
        } else {
            queue.get_or_create_topic_id(topic_name);
            true
        }
    }
    
    /// Register a topic and get its ID (handle) for fast publishing
    /// Returns the topic ID that can be used with publish_by_id
    pub fn register_topic(&self, topic_name: &str) -> u32 {
        self.inner.borrow_mut().get_or_create_topic_id(topic_name) as u32
    }

    pub fn destroy_topic(&self, topic_id: u32) -> bool {
        let mut queue = self.inner.borrow_mut();
        if let Some(topic) = queue.topics.get_mut(topic_id as usize) {
            topic.subscribers.clear();
            topic.disable_buffer();
            return true;
        }
        false
    }

    /// Subscribe to a topic using its ID
    /// Callback signature: (payload, topic_id, timestamp, message_id)
    pub fn subscribe(&self, topic_id: u32, callback: Function) -> Result<u32, JsValue> {
        let mut queue = self.inner.borrow_mut();
        
        if let Some(topic) = queue.topics.get_mut(topic_id as usize) {
            let sub_id = topic.next_id;
            topic.subscribers.insert(sub_id, callback);
            topic.next_id = topic.next_id.wrapping_add(1);
            Ok(sub_id)
        } else {
            Err(JsValue::from_str("Invalid topic ID"))
        }
    }

    /// Unsubscribe from a topic using its ID
    pub fn unsubscribe(&self, topic_id: u32, sub_id: u32) -> bool {
        let mut queue = self.inner.borrow_mut();
        if let Some(topic) = queue.topics.get_mut(topic_id as usize) {
            topic.subscribers.remove(&sub_id).is_some()
        } else {
            false
        }
    }
    
    /// Publish using a topic ID (handle)
    /// This is O(1) and avoids string hashing/copying - significantly faster for high frequency
    pub fn publish(&self, topic_id: u32, payload: JsValue) -> Result<(), JsValue> {
        // Step 1: Create message and dispatch locally
        let (msg_js, has_channel) = {
            let mut queue = self.inner.borrow_mut();
            
            // Verify topic ID exists
            if topic_id as usize >= queue.topics.len() {
                return Err(JsValue::from_str("Invalid topic ID"));
            }

            // Use a simple counter for ID or generate UUID if needed?
            // For now, let's use a random u64 which is faster than string UUID
            // Or better: use a simple counter for local messages
            let id = (js_sys::Math::random() * 1e16) as u64;

            let message = Message {
                id,
                topic_id,
                payload,
                timestamp: js_sys::Date::now(),
                origin_id: queue.client_id.clone(),
            };
            
            let rc_msg = Rc::new(message);

            // Dispatch locally
            // No JS object creation needed here for local dispatch!
            queue.dispatch_local(&rc_msg, None);

            // Check if we have a channel
            let has_channel = queue.channel.is_some();
            
            // Only create JS object if we really need to broadcast
            let msg_js = if has_channel {
                // For broadcast we still need the old object format or a new one?
                // Let's stick to the old format for now for compatibility with other tabs
                // But we need the topic name
                let topic_name = &queue.topics[topic_id as usize].name;
                let raw_msg = crate::js_utils::message_to_js(&rc_msg, topic_name)?;
                
                // Wrap in packet [0, msg] for protocol
                let packet = Array::new();
                packet.push(&JsValue::from(0));
                packet.push(&raw_msg);
                packet.into()
            } else {
                JsValue::NULL
            };

            (msg_js, has_channel)
        };

        // Step 2: Broadcast if channel exists
        if has_channel {
            let queue = self.inner.borrow();
            if let Some(ref channel) = queue.channel {
                channel.post_message(&msg_js).map_err(|_| {
                    JsValue::from_str("Failed to broadcast message")
                })?;
            }
        }

        Ok(())
    }

    /// Publish a message asynchronously using Promise/microtask
    /// This returns immediately and delivers the message in the next microtask
    /// Useful for non-blocking operations and better browser responsiveness
    pub fn publish_async(&self, topic_id: u32, payload: JsValue) -> Result<Promise, JsValue> {
        // Clone necessary data for the async closure
        let inner = self.inner.clone();
        let payload_clone = payload.clone();

        // Create a Promise that resolves in a microtask
        let promise = Promise::new(&mut |resolve, reject| {
            // Schedule in a microtask using Promise.resolve().then()
            let resolve_clone = resolve.clone();
            let reject_clone = reject.clone();

            // Clone again for the inner closure
            let inner2 = inner.clone();
            let payload_clone2 = payload_clone.clone();

            // Create the closure that will run in the microtask
            let closure = Closure::once(move |_value: JsValue| {
                let mut queue = match inner2.try_borrow_mut() {
                    Ok(q) => q,
                    Err(_) => {
                        let err_msg = JsValue::from_str("Failed to borrow queue");
                        let _ = reject_clone.call1(&JsValue::NULL, &err_msg);
                        return;
                    }
                };

                // Generate u64 ID
                let id = (js_sys::Math::random() * 1e16) as u64;
                
                // Verify topic ID exists
                if (topic_id as usize) >= queue.topics.len() {
                     let err_msg = JsValue::from_str("Invalid topic ID");
                     let _ = reject_clone.call1(&JsValue::NULL, &err_msg);
                     return;
                }

                // Create the message
                let message = Message {
                    id,
                    topic_id,
                    payload: payload_clone2,
                    timestamp: js_sys::Date::now(),
                    origin_id: queue.client_id.clone(),
                };
                
                let rc_msg = Rc::new(message);
                
                // Dispatch locally
                queue.dispatch_local(&rc_msg, None);

                // Resolve the promise
                let _ = resolve_clone.call0(&JsValue::NULL);
            });

            // Use Promise.resolve().then() to create a microtask
            let microtask_promise = Promise::resolve(&JsValue::UNDEFINED);
            
            // Pass the function to then() and forget the closure to keep it alive
            let _ = microtask_promise.then(&closure);
            
            // IMPORTANT: We must forget the closure so it's not dropped before execution
            closure.forget();
        });

        Ok(promise)
    }

    pub fn get_client_id(&self) -> String {
        self.inner.borrow().client_id.as_ref().clone()
    }

    /// Get the number of topics
    pub fn topic_count(&self) -> usize {
        self.inner.borrow().topics.len()
    }

    /// Get the number of subscribers for a specific topic
    pub fn subscriber_count(&self, topic_id: u32) -> usize {
        self.inner.borrow()
            .get_topic_by_id(topic_id as usize)
            .map_or(0, |topic| topic.subscribers.len())
    }

    /// Check if a topic exists
    pub fn has_topic(&self, topic_id: u32) -> bool {
        self.has_topic_id(topic_id)
    }

    /// Unsubscribe all subscribers from a topic
    pub fn unsubscribe_all(&self, topic_id: u32) -> usize {
        let mut queue = self.inner.borrow_mut();
        if let Some(topic) = queue.get_topic_by_id_mut(topic_id as usize) {
            let count = topic.subscribers.len();
            topic.subscribers.clear();
            count
        } else {
            0
        }
    }

    /// Enable message buffering for a specific topic
    /// Messages will be cached in a ring buffer for later retrieval
    /// @param topic_id - ID of the topic
    /// @param capacity - Maximum number of messages to buffer (default: 100)
    #[wasm_bindgen]
    pub fn enable_topic_buffer(&self, topic_id: u32, capacity: Option<usize>) -> Result<(), JsValue> {
        let cap = capacity.unwrap_or(100);
        if cap == 0 {
            return Err(JsValue::from_str("Buffer capacity must be greater than 0"));
        }

        let mut queue = self.inner.borrow_mut();
        if let Some(topic) = queue.get_topic_by_id_mut(topic_id as usize) {
            topic.enable_buffer(cap);
            Ok(())
        } else {
            Err(JsValue::from_str("Invalid topic ID"))
        }
    }

    /// Disable message buffering for a specific topic
    /// Clears all buffered messages
    /// @param topic_id - ID of the topic
    #[wasm_bindgen]
    pub fn disable_topic_buffer(&self, topic_id: u32) -> Result<(), JsValue> {
        let mut queue = self.inner.borrow_mut();
        if let Some(topic) = queue.get_topic_by_id_mut(topic_id as usize) {
            topic.disable_buffer();
            Ok(())
        } else {
            Err(JsValue::from_str("Invalid topic ID"))
        }
    }

    /// Get the current size of the message buffer for a topic
    /// @param topic_id - ID of the topic
    /// @returns Number of messages currently buffered, or -1 if buffering is not enabled
    #[wasm_bindgen]
    pub fn get_buffer_size(&self, topic_id: u32) -> i32 {
        let queue = self.inner.borrow();
        if let Some(topic) = queue.get_topic_by_id(topic_id as usize) {
            topic.get_buffer()
                .map(|b| b.len() as i32)
                .unwrap_or(-1)
        } else {
            -1
        }
    }

    /// Get the buffer capacity for a topic
    /// @param topic_id - ID of the topic
    /// @returns Maximum buffer capacity, or 0 if buffering is not enabled
    #[wasm_bindgen]
    pub fn get_buffer_capacity(&self, topic_id: u32) -> usize {
        let queue = self.inner.borrow();
        if let Some(topic) = queue.get_topic_by_id(topic_id as usize) {
            topic.get_buffer()
                .map(|b| b.capacity())
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Clear all buffered messages for a topic
    /// @param topic_id - ID of the topic
    /// @returns Number of messages cleared
    #[wasm_bindgen]
    pub fn clear_buffer(&self, topic_id: u32) -> usize {
        let mut queue = self.inner.borrow_mut();
        if let Some(topic) = queue.get_topic_by_id_mut(topic_id as usize) {
            if let Some(buffer) = topic.get_buffer_mut() {
                let count = buffer.len();
                buffer.clear();
                count
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Check if a topic has buffering enabled
    /// @param topic_id - ID of the topic
    #[wasm_bindgen]
    pub fn has_buffer(&self, topic_id: u32) -> bool {
        let queue = self.inner.borrow();
        if let Some(topic) = queue.get_topic_by_id(topic_id as usize) {
            topic.has_buffer()
        } else {
            false
        }
    }

    /// Get buffered messages for a topic as a JavaScript array
    /// @param topic_id - ID of the topic
    /// @returns Array of buffered messages (oldest first), or empty array if no buffer
    #[wasm_bindgen]
    pub fn get_buffered_messages(&self, topic_id: u32) -> Result<js_sys::Array, JsValue> {
        let queue = self.inner.borrow();
        if let Some(topic) = queue.get_topic_by_id(topic_id as usize) {
            if let Some(buffer) = topic.get_buffer() {
                let messages = buffer.to_vec();
                let array = js_sys::Array::new();
                for msg in messages {
                    let msg_js = crate::js_utils::message_to_js(&msg, &topic.name)?;
                    array.push(&msg_js);
                }
                Ok(array)
            } else {
                Ok(js_sys::Array::new())
            }
        } else {
            Ok(js_sys::Array::new())
        }
    }

    pub fn close(&mut self) -> Result<(), JsValue> {
        let mut queue = self.inner.borrow_mut();
        if let Some(channel) = &queue.channel {
            channel.close();
            channel.set_onmessage(None);
        }
        queue.channel = None;
        queue.topics.clear();
        queue.topic_index.clear();

        // Clear the closure - it will be properly dropped here
        self._closure.take();

        Ok(())
    }

    /// Publish multiple messages by ID (handle) efficiently
    /// This is the fastest way to publish multiple messages
    pub fn publish_batch_by_id(&self, topic_id: u32, payloads: js_sys::Array) -> Result<(), JsValue> {
        // No need to lookup name anymore, just use ID
        // Verify ID exists once
        if !self.has_topic_id(topic_id) {
             return Err(JsValue::from_str("Invalid topic ID"));
        }

        for payload in payloads.iter() {
            self.publish(topic_id, payload)?;
        }
        Ok(())
    }

    fn has_topic_id(&self, topic_id: u32) -> bool {
        let queue = self.inner.borrow();
        (topic_id as usize) < queue.topics.len()
    }
}

// ============================================================================
// Additional MessageQueue Methods (Not exported to JavaScript)
// ============================================================================

impl MessageQueue {
    /// Publish multiple messages efficiently
    /// Note: Not exported to JavaScript due to wasm-bindgen limitations with tuple vectors.
    /// Use multiple publish() calls instead from JS code.
    pub fn publish_batch(&self, messages: Vec<(u32, JsValue)>) -> Result<(), JsValue> {
        for (topic_id, payload) in messages {
            self.publish(topic_id, payload)?;
        }
        Ok(())
    }
}

// Implement Default trait for convenience
impl Default for MessageQueue {
    fn default() -> Self {
        Self::new(None).expect("Failed to create default MessageQueue")
    }
}

// Implement Drop trait for automatic resource cleanup
impl Drop for MessageQueue {
    fn drop(&mut self) {
        // Close the broadcast channel and clear resources
        if let Ok(mut queue) = self.inner.try_borrow_mut() {
            if let Some(channel) = &queue.channel {
                channel.close();
                channel.set_onmessage(None);
            }
            queue.channel = None;
            queue.topics.clear();
        }

        // The closure will be automatically dropped when _closure is dropped
        // No need to explicitly call forget() as Drop handles cleanup
    }
}
