use js_sys::JsString;
use js_sys::JSON::{parse, stringify};
use wasm_bindgen::prelude::JsValue;

use usdpl_core::serdes::Primitive;

pub(crate) fn primitive_to_js(primitive: Primitive) -> JsValue {
    match primitive {
        Primitive::Empty => JsValue::null(),
        Primitive::String(s) => JsValue::from_str(&s),
        Primitive::F32(f) => JsValue::from_f64(f as _),
        Primitive::F64(f) => JsValue::from_f64(f),
        Primitive::U32(f) => JsValue::from_f64(f as _),
        Primitive::U64(f) => JsValue::from_f64(f as _),
        Primitive::I32(f) => JsValue::from_f64(f as _),
        Primitive::I64(f) => JsValue::from_f64(f as _),
        Primitive::Bool(b) => JsValue::from_bool(b),
        Primitive::Json(s) => parse(&s).ok().unwrap_or(JsValue::from_str(&s)),
    }
}

pub(crate) fn js_to_primitive(val: JsValue) -> Primitive {
    if let Some(b) = val.as_bool() {
        Primitive::Bool(b)
    } else if let Some(f) = val.as_f64() {
        Primitive::F64(f)
    } else if let Some(s) = val.as_string() {
        Primitive::String(s)
    } else if val.is_null() || val.is_undefined() {
        Primitive::Empty
    } else if let Ok(s) = stringify(&val) {
        Primitive::Json(s.as_string().unwrap())
    } else {
        Primitive::Empty
    }
}

pub(crate) fn str_to_js<S: std::string::ToString>(s: S) -> JsString {
    s.to_string().into()
}

pub(crate) fn js_to_str(js: JsValue) -> String {
    if let Some(s) = js.as_string() {
        s
    } else {
        format!("{:?}", js)
    }
}
