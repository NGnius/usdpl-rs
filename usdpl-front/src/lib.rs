//! Front-end library to be called from Javascript.
//! Targets WASM.
//!
//! In true Javascript tradition, this part of the library does not support error handling.
//!
#![warn(missing_docs)]

mod client_handler;
pub use client_handler::WebSocketHandler;
mod connection;
mod convert;
mod imports;
pub mod wasm;

/*#[allow(missing_docs)] // existence is pain otherwise
pub mod _nrpc_js_interop {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}*/

#[allow(missing_docs)]
pub mod _helpers {
    pub use js_sys;
    pub use wasm_bindgen;
    pub use wasm_bindgen_futures;
    pub use log;
}

use std::sync::atomic::{AtomicU64, Ordering};

use js_sys::Array;
use wasm_bindgen::prelude::*;

use usdpl_core::{socket::Packet, RemoteCall};
//const REMOTE_CALL_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
//const REMOTE_PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(31337);

static mut CTX: UsdplContext = UsdplContext {
    port: 0,
    id: AtomicU64::new(0),
    #[cfg(feature = "encrypt")]
    key: Vec::new(),
};

static mut CACHE: Option<std::collections::HashMap<String, JsValue>> = None;

static mut TRANSLATIONS: Option<std::collections::HashMap<String, Vec<String>>> = None;

#[cfg(feature = "encrypt")]
fn encryption_key() -> Vec<u8> {
    hex::decode(obfstr::obfstr!(env!("USDPL_ENCRYPTION_KEY"))).unwrap()
}

//#[wasm_bindgen]
#[derive(Debug)]
struct UsdplContext {
    port: u16,
    id: AtomicU64,
    #[cfg(feature = "encrypt")]
    key: Vec<u8>,
}

fn get_port() -> u16 {
    unsafe { CTX.port }
}

#[cfg(feature = "encrypt")]
fn get_key() -> Vec<u8> {
    unsafe { CTX.key.clone() }
}

fn increment_id() -> u64 {
    let atomic = unsafe { &CTX.id };
    atomic.fetch_add(1, Ordering::SeqCst)
}

/// Initialize the front-end library
#[wasm_bindgen]
pub fn init_usdpl(port: u16) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    //REMOTE_PORT.store(port, std::sync::atomic::Ordering::SeqCst);
    unsafe {
        CTX = UsdplContext {
            port: port,
            id: AtomicU64::new(0),
            #[cfg(feature = "encrypt")]
            key: encryption_key(),
        };
    }

    unsafe {
        CACHE = Some(std::collections::HashMap::new());
    }
}

/// Get the targeted plugin framework, or "any" if unknown
#[wasm_bindgen]
pub fn target_usdpl() -> String {
    usdpl_core::api::Platform::current().to_string()
}

/// Get the UDSPL front-end version
#[wasm_bindgen]
pub fn version_usdpl() -> String {
    env!("CARGO_PKG_VERSION").into()
}

/// Get the targeted plugin framework, or "any" if unknown
#[wasm_bindgen]
pub fn set_value(key: String, value: JsValue) -> JsValue {
    unsafe {
        CACHE
            .as_mut()
            .unwrap()
            .insert(key, value)
            .unwrap_or(JsValue::NULL)
    }
}

/// Get the targeted plugin framework, or "any" if unknown
#[wasm_bindgen]
pub fn get_value(key: String) -> JsValue {
    unsafe {
        CACHE
            .as_ref()
            .unwrap()
            .get(&key)
            .map(|x| x.to_owned())
            .unwrap_or(JsValue::UNDEFINED)
    }
}

/// Call a function on the back-end.
/// Returns null (None) if this fails for any reason.
#[wasm_bindgen]
pub async fn call_backend(name: String, parameters: Vec<JsValue>) -> JsValue {
    #[cfg(feature = "debug")]
    imports::console_log(&format!(
        "call_backend({}, [params; {}])",
        name,
        parameters.len()
    ));
    let next_id = increment_id();
    let mut params = Vec::with_capacity(parameters.len());
    for val in parameters {
        params.push(convert::js_to_primitive(val));
    }
    let port = get_port();
    #[cfg(feature = "debug")]
    imports::console_log(&format!("USDPL: Got port {}", port));
    let results = connection::send_call(
        next_id,
        Packet::Call(RemoteCall {
            id: next_id,
            function: name.clone(),
            parameters: params,
        }),
        port,
        #[cfg(feature = "encrypt")]
        get_key(),
    )
    .await;
    let results = match results {
        Ok(x) => x,
        #[allow(unused_variables)]
        Err(e) => {
            #[cfg(feature = "debug")]
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

/// Initialize translation strings for the front-end
#[wasm_bindgen]
pub async fn init_tr(locale: String) {
    let next_id = increment_id();
    match connection::send_recv_packet(
        next_id,
        Packet::Language(locale.clone()),
        get_port(),
        #[cfg(feature = "encrypt")]
        get_key(),
    )
    .await
    {
        Ok(Packet::Translations(translations)) => {
            #[cfg(feature = "debug")]
            imports::console_log(&format!("USDPL: Got translations for {}", locale));
            // convert translations into map
            let mut tr_map = std::collections::HashMap::with_capacity(translations.len());
            for (key, val) in translations {
                tr_map.insert(key, val);
            }
            unsafe { TRANSLATIONS = Some(tr_map) }
        }
        Ok(_) => {
            #[cfg(feature = "debug")]
            imports::console_error(&format!("USDPL: Got wrong packet response for init_tr"));
            unsafe { TRANSLATIONS = None }
        }
        #[allow(unused_variables)]
        Err(e) => {
            #[cfg(feature = "debug")]
            imports::console_error(&format!("USDPL: Got wrong error for init_tr: {:#?}", e));
            unsafe { TRANSLATIONS = None }
        }
    }
}

/// Translate a phrase, equivalent to tr_n(msg_id, 0)
#[wasm_bindgen]
pub fn tr(msg_id: String) -> String {
    if let Some(translations) = unsafe { TRANSLATIONS.as_ref().unwrap().get(&msg_id) } {
        if let Some(translated) = translations.get(0) {
            translated.to_owned()
        } else {
            msg_id
        }
    } else {
        msg_id
    }
}

/// Translate a phrase, retrieving the plural form for `n` items
#[wasm_bindgen]
pub fn tr_n(msg_id: String, n: usize) -> String {
    if let Some(translations) = unsafe { TRANSLATIONS.as_ref().unwrap().get(&msg_id) } {
        if let Some(translated) = translations.get(n) {
            translated.to_owned()
        } else {
            msg_id
        }
    } else {
        msg_id
    }
}
