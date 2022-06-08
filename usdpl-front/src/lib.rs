//! Front-end library to be called from Javascript.
//! Targets WASM.
//!
//! In true Javascript tradition, this part of the library does not support error handling.
//!

mod connection;

use wasm_bindgen::prelude::*;
use js_sys::JSON::{stringify, parse};

use usdpl_core::{socket::Packet, RemoteCall, serdes::Primitive};
const REMOTE_CALL_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

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
pub fn init() -> bool {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    true
}

/// Get the targeted plugin framework, or "any" if unknown
#[wasm_bindgen]
pub fn target() -> String {
    "any".to_string()
}

/// Call a function on the back-end.
/// Returns null (None) if this fails for any reason.
#[wasm_bindgen]
pub fn call_backend(name: String, parameters: Vec<JsValue>) -> Option<Vec<JsValue>> {
    let next_id = REMOTE_CALL_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut params = Vec::with_capacity(parameters.len());
    for val in parameters {
        if let Some(b) = val.as_bool() {
            params.push(Primitive::Bool(b));
        } else if let Some(f) = val.as_f64() {
            params.push(Primitive::F64(f));
        } else if let Some(s) = val.as_string() {
            params.push(Primitive::String(s));
        } else if val.is_null() || val.is_undefined() {
            params.push(Primitive::Empty);
        } else if let Ok(s) = stringify(&val) {
            params.push(Primitive::Json(s.as_string().unwrap()));
        } else {
            return None;
        }
    }
    let results = match connection::send_native(Packet::Call(RemoteCall {
        id: next_id,
        function: name,
        parameters: params,
    })) {
        Some(Packet::CallResponse(resp)) => resp,
        _ => return None,
    };
    let mut js_results = Vec::with_capacity(results.response.len());
    for val in results.response {
        let js_val = match val {
            Primitive::Empty => JsValue::null(),
            Primitive::String(s) => JsValue::from_str(&s),
            Primitive::F32(f)=> JsValue::from_f64(f as _),
            Primitive::F64(f)=> JsValue::from_f64(f),
            Primitive::U32(f)=> JsValue::from_f64(f as _),
            Primitive::U64(f)=> JsValue::from_f64(f as _),
            Primitive::I32(f)=> JsValue::from_f64(f as _),
            Primitive::I64(f)=> JsValue::from_f64(f as _),
            Primitive::Bool(b) => JsValue::from_bool(b),
            Primitive::Json(s) => parse(&s).ok().unwrap_or(JsValue::from_str(&s)),
        };
        js_results.push(js_val);
    }
    Some(js_results)
}
