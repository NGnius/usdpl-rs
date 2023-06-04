use std::collections::HashMap;

use js_sys::Map;

use super::{FromWasmable, IntoWasmable};

macro_rules! numbers_map {
    ($num_ty: ident) => {
        impl FromWasmable<Map> for HashMap<String, $num_ty> {
            fn from_wasm(js: Map) -> Self {
                let mut result = HashMap::with_capacity(js.size() as usize);
                js.for_each(&mut |key, val| {
                    if let Some(key) = key.as_string() {
                        if let Some(val) = val.as_f64() {
                            result.insert(key, val as $num_ty);
                        }
                    }
                });
                result
            }
        }

        impl IntoWasmable<Map> for HashMap<String, $num_ty> {
            fn into_wasm(self) -> Map {
                let result = Map::new();
                for (key, val) in self {
                    result.set(&key.into(), &val.into());
                }
                result
            }
        }
    };
}

numbers_map! { f64 }
numbers_map! { f32 }

numbers_map! { isize }
numbers_map! { usize }

numbers_map! { i8 }
numbers_map! { i16 }
numbers_map! { i32 }
numbers_map! { i64 }
numbers_map! { i128 }

numbers_map! { u8 }
numbers_map! { u16 }
numbers_map! { u32 }
numbers_map! { u64 }
numbers_map! { u128 }

impl FromWasmable<Map> for HashMap<String, String> {
    fn from_wasm(js: Map) -> Self {
        let mut result = HashMap::with_capacity(js.size() as usize);
        js.for_each(&mut |key, val| {
            if let Some(key) = key.as_string() {
                if let Some(val) = val.as_string() {
                    result.insert(key, val);
                }
            }
        });
        result
    }
}

impl IntoWasmable<Map> for HashMap<String, String> {
    fn into_wasm(self) -> Map {
        let result = Map::new();
        for (key, val) in self {
            result.set(&key.into(), &val.into());
        }
        result
    }
}

impl FromWasmable<Map> for HashMap<String, bool> {
    fn from_wasm(js: Map) -> Self {
        let mut result = HashMap::with_capacity(js.size() as usize);
        js.for_each(&mut |key, val| {
            if let Some(key) = key.as_string() {
                if let Some(val) = val.as_bool() {
                    result.insert(key, val);
                }
            }
        });
        result
    }
}

impl IntoWasmable<Map> for HashMap<String, bool> {
    fn into_wasm(self) -> Map {
        let result = Map::new();
        for (key, val) in self {
            result.set(&key.into(), &val.into());
        }
        result
    }
}
