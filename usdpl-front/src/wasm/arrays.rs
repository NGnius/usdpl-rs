use js_sys::Array;

use super::{FromWasmable, IntoWasmable};

macro_rules! numbers_array {
    ($num_ty: ident) => {
        impl FromWasmable<Array> for Vec<$num_ty> {
            fn from_wasm(js: Array) -> Self {
                let mut result = Vec::with_capacity(js.length() as usize);
                js.for_each(&mut |val, _index, _arr| {
                    // according to MDN, this is guaranteed to be in order so index can be ignored
                    if let Some(val) = val.as_f64() {
                        result.push(val as $num_ty);
                    }
                });
                result
            }
        }

        impl IntoWasmable<Array> for Vec<$num_ty> {
            fn into_wasm(self) -> Array {
                let result = Array::new();
                for val in self {
                    result.push(&val.into());
                }
                result
            }
        }
    };
}

numbers_array! { f64 }
numbers_array! { f32 }

numbers_array! { isize }
numbers_array! { usize }

numbers_array! { i8 }
numbers_array! { i16 }
numbers_array! { i32 }
numbers_array! { i64 }
numbers_array! { i128 }

numbers_array! { u8 }
numbers_array! { u16 }
numbers_array! { u32 }
numbers_array! { u64 }
numbers_array! { u128 }

impl FromWasmable<Array> for Vec<String> {
    fn from_wasm(js: Array) -> Self {
        let mut result = Vec::with_capacity(js.length() as usize);
        js.for_each(&mut |val, _index, _arr| {
            // according to MDN, this is guaranteed to be in order so index can be ignored
            if let Some(val) = val.as_string() {
                result.push(val);
            }
        });
        result
    }
}

impl IntoWasmable<Array> for Vec<String> {
    fn into_wasm(self) -> Array {
        let result = Array::new();
        for val in self {
            result.push(&val.into());
        }
        result
    }
}

impl FromWasmable<Array> for Vec<bool> {
    fn from_wasm(js: Array) -> Self {
        let mut result = Vec::with_capacity(js.length() as usize);
        js.for_each(&mut |val, _index, _arr| {
            // according to MDN, this is guaranteed to be in order so index can be ignored
            if let Some(val) = val.as_bool() {
                result.push(val);
            }
        });
        result
    }
}

impl IntoWasmable<Array> for Vec<bool> {
    fn into_wasm(self) -> Array {
        let result = Array::new();
        for val in self {
            result.push(&val.into());
        }
        result
    }
}
