//! Front-end library to be called from Javascript.
//! Targets WASM.
//!
//! In true Javascript tradition, this part of the library does not support error handling.
//!

mod connection;
mod convert;

use wasm_bindgen::prelude::*;

use usdpl_core::{socket::Packet, RemoteCall};
const REMOTE_CALL_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
const REMOTE_PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(31337);

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    //fn alert(s: &str);
}

/// Initialize the front-end library
#[wasm_bindgen]
pub fn init_usdpl(port: u16) -> bool {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    REMOTE_PORT.store(port, std::sync::atomic::Ordering::Relaxed);
    true
}

/// Get the targeted plugin framework, or "any" if unknown
#[wasm_bindgen]
pub fn target() -> String {
    #[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
    {"decky".to_string()}
    #[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
    {"crankshaft".to_string()}
    #[cfg(not(any(feature = "decky", feature = "crankshaft")))]
    {"any".to_string()}
}

/// Call a function on the back-end.
/// Returns null (None) if this fails for any reason.
#[wasm_bindgen]
pub fn call_backend(name: String, parameters: Vec<JsValue>) -> Option<Vec<JsValue>> {
    let next_id = REMOTE_CALL_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut params = Vec::with_capacity(parameters.len());
    for val in parameters {
        params.push(convert::js_to_primitive(val));
    }
    let results = match connection::send_native(Packet::Call(RemoteCall {
        id: next_id,
        function: name,
        parameters: params,
    }), REMOTE_PORT.load(std::sync::atomic::Ordering::Relaxed)) {
        Some(Packet::CallResponse(resp)) => resp,
        _ => return None,
    };
    let mut js_results = Vec::with_capacity(results.response.len());
    for val in results.response {
        let js_val = convert::primitive_to_js(val);
        js_results.push(js_val);
    }
    Some(js_results)
}
