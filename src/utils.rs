use wasm_bindgen::prelude::*;
use crate::constants::ERR_WINDOW_NOT_AVAILABLE;
use crate::constants::ERR_CRYPTO_NOT_AVAILABLE;
use std::cell::RefCell;

thread_local! {
    static CRYPTO: RefCell<Option<web_sys::Crypto>> = RefCell::new(None);
}

/// Generate a UUID using the browser's crypto API
pub fn generate_uuid() -> Result<String, JsValue> {
    CRYPTO.with(|crypto_cell| {
        let mut crypto_opt = crypto_cell.borrow_mut();
        
        if let Some(crypto) = crypto_opt.as_ref() {
            return Ok(crypto.random_uuid());
        }

        let window = web_sys::window()
            .ok_or_else(|| JsValue::from_str(ERR_WINDOW_NOT_AVAILABLE))?;

        let crypto = window.crypto()
            .map_err(|_| JsValue::from_str(ERR_CRYPTO_NOT_AVAILABLE))?;

        let uuid = crypto.random_uuid();
        *crypto_opt = Some(crypto);
        
        Ok(uuid)
    })
}
