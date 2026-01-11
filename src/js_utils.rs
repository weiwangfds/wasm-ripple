use wasm_bindgen::prelude::*;
use js_sys::{Function, Array};
use std::rc::Rc;
use crate::types::Message;

thread_local! {
    static MSG_FACTORY: Function = Function::new_with_args(
        "id, topic, payload, timestamp, origin_id",
        "return {id: id, topic: topic, payload: payload, timestamp: timestamp, origin_id: origin_id};"
    );
    
    static MSG_EXTRACTOR: Function = Function::new_with_args(
        "obj",
        "return [obj.id, obj.topic, obj.payload, obj.timestamp, obj.origin_id];"
    );
}

/// Convert a Message struct to a JavaScript object
/// Kept for BroadcastChannel compatibility
pub fn message_to_js(msg: &Message, topic_name: &str) -> Result<JsValue, JsValue> {
    MSG_FACTORY.with(|factory| {
        // Use explicit type annotations to help rust-analyzer
        let id_val: JsValue = JsValue::from(msg.id as f64);
        let topic_val: JsValue = topic_name.into();
        let timestamp_val: JsValue = msg.timestamp.into();
        let origin_id_val: JsValue = msg.origin_id.as_str().into();

        factory.call5(
            &JsValue::NULL,
            &id_val,
            &topic_val,
            &msg.payload,
            &timestamp_val,
            &origin_id_val
        )
    })
}

/// Parse a JavaScript object into a Message struct
/// Returns (Message, String) tuple where String is the topic name
pub fn parse_js_message(val: &JsValue) -> Result<(Message, String), JsValue> {
    MSG_EXTRACTOR.with(|extractor| {
        let arr_val = extractor.call1(&JsValue::NULL, val)?;
        let arr = Array::from(&arr_val);

        // Helper to check for undefined/null
        let check_val = |v: JsValue, name: &str| -> Result<JsValue, JsValue> {
            if v.is_undefined() || v.is_null() {
                Err(JsValue::from_str(&format!("Missing field: {}", name)))
            } else {
                Ok(v)
            }
        };

        // ID can be string or number from JS
        let id_val = check_val(arr.get(0), "id")?;
        let id = if let Some(n) = id_val.as_f64() {
            n as u64
        } else if let Some(s) = id_val.as_string() {
            // Try parsing string to u64, fallback to hash if needed, or 0
            // For now assuming it's a number if coming from our system
            s.parse::<u64>().unwrap_or(0)
        } else {
            return Err(JsValue::from_str("Invalid id type"));
        };
            
        let topic_name = check_val(arr.get(1), "topic")?.as_string()
            .ok_or_else(|| JsValue::from_str("Invalid topic type"))?;
            
        let payload = arr.get(2); // Payload can be anything
        
        let timestamp = check_val(arr.get(3), "timestamp")?.as_f64()
            .ok_or_else(|| JsValue::from_str("Invalid timestamp type"))?;
            
        let origin_id = check_val(arr.get(4), "origin_id")?.as_string()
            .ok_or_else(|| JsValue::from_str("Invalid origin_id type"))?;

        // Note: topic_id will be resolved by the caller using topic_name
        Ok((Message {
            id,
            topic_id: 0, // Placeholder, must be filled by caller
            payload,
            timestamp,
            origin_id: Rc::new(origin_id),
        }, topic_name))
    })
}
