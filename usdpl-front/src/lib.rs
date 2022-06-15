//! Front-end library to be called from Javascript.
//! Targets WASM.
//!
//! In true Javascript tradition, this part of the library does not support error handling.
//!

mod connection;
mod convert;
mod imports;

use wasm_bindgen::prelude::*;
use js_sys::Array;

use usdpl_core::{socket::Packet, RemoteCall};
//const REMOTE_CALL_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
//const REMOTE_PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(31337);

static mut CTX: UsdplContext = UsdplContext {
    port: 31337,
    id: 1,
};

#[wasm_bindgen]
#[derive(Debug)]
pub struct UsdplContext {
    port: u16,
    id: u64,
}

fn get_port() -> u16 {
    unsafe {CTX.port}
}

fn increment_id() -> u64 {
    let current_id = unsafe {CTX.id};
    unsafe {CTX.id += 1;}
    current_id
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize the front-end library
#[wasm_bindgen]
pub fn init_usdpl(port: u16) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    //REMOTE_PORT.store(port, std::sync::atomic::Ordering::SeqCst);
    unsafe {
        CTX = UsdplContext {
            port: port,
            id: 1,
        };
    }
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
pub async fn call_backend(name: String, parameters: Vec<JsValue>) -> JsValue {
    imports::console_log(&format!("call_backend({}, [params; {}])", name, parameters.len()));
    let next_id = increment_id();
    let mut params = Vec::with_capacity(parameters.len());
    for val in parameters {
        params.push(convert::js_to_primitive(val));
    }
    let port = get_port();
    imports::console_log(&format!("USDPL: Got port {}", port));
    let results = connection::send_js(Packet::Call(RemoteCall {
        id: next_id,
        function: name.clone(),
        parameters: params,
    }), port).await;
    let results = match results {
        Ok(x) => x,
        Err(e) => {
            imports::console_error(&format!("USDPL: Got error while calling {}: {:?}", name, e));
            return JsValue::NULL;
        }
    };
    let results_js = Array::new_with_length(results.len() as _);
    let mut i = 0;
    for item in results {
        results_js.set(i as _, convert::primitive_to_js(item));
        i += 1;
    }
    results_js.into()
}
