use crate::types::{InnerQueue, Message};
use wasm_bindgen::JsValue;
use std::rc::Rc;
// use web_sys::console; // Removed for size optimization

impl InnerQueue {
    /// Dispatch a message to all local subscribers of its topic
    /// 
    /// # Arguments
    /// * `message` - The message to dispatch (wrapped in Rc for efficient cloning)
    /// * `_msg_js` - Deprecated/Unused. We now pass arguments directly.
    pub fn dispatch_local(&mut self, message: &Rc<Message>, _msg_js: Option<&JsValue>) {
        // Try to find topic by index
        // Since message.topic_id is a u32, we can directly use it as index
        // But we need to verify it's valid
        let topic_idx = message.topic_id as usize;
        
        // Safety check: ensure topic_idx is within bounds
        if topic_idx >= self.topics.len() {
             return;
        }
        
        if let Some(topic) = self.topics.get_mut(topic_idx) {
            // Store message in buffer if buffering is enabled
            if let Some(buffer) = topic.get_buffer_mut() {
                buffer.push(message.clone());
            }

            // Optimization: Zero-allocation dispatch
            // Instead of creating a JS object, we pass arguments directly to the callback.
            // Signature: callback(payload, topic_id, timestamp, id)
            // This avoids Reflect::set/get and object creation entirely.
            
            let this = JsValue::NULL;
            let topic_id_val = JsValue::from(message.topic_id);
            let timestamp_val = JsValue::from(message.timestamp);
            // ID is u64, precision loss in JS Number (f64) is possible for values > 2^53
            // But for our random usage it's fine, or we pass as BigInt if needed.
            // For speed, let's pass as f64.
            let id_val = JsValue::from(message.id as f64);

            for sub in topic.subscribers.values() {
                // call4 is faster than creating an array or object
                if let Err(_) = sub.call4(&this, &message.payload, &topic_id_val, &timestamp_val, &id_val) {
                    // console::error_1(&e); // Removed console logging for size optimization
                }
            }
        }
    }
}


