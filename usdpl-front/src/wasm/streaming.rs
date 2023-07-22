use wasm_bindgen::JsValue;

/// Convert Rust type to WASM-compatible type involved in nRPC streaming
pub trait IntoWasmStreamableType {
    /// Required method
    fn into_wasm_streamable(self) -> JsValue;
}

#[derive(Debug)]
/// Conversion error from FromWasmStreamableType
pub enum WasmStreamableConversionError {
    /// JSValue underlying type is incorrect
    UnexpectedType {
        /// Expected Javascript type
        expected: JsType,
        /// Actual Javascript type
        got: JsType,
    },
}

impl core::fmt::Display for WasmStreamableConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedType { expected, got } => write!(f, "Unexpected type {}, expected {}", expected, got),
        }
    }
}

impl std::error::Error for WasmStreamableConversionError {}

/// Approximation of all possible JS types detectable through Wasm
#[allow(missing_docs)]
#[derive(Debug)]
pub enum JsType {
    Number,
    String,
    Bool,
    Array,
    BigInt,
    Function,
    Symbol,
    Undefined,
    Null,
    Object,
    Unknown,
}

impl core::fmt::Display for JsType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Number => write!(f, "number"),
            Self::String => write!(f, "string"),
            Self::Bool => write!(f, "boolean"),
            Self::Array => write!(f, "array"),
            Self::BigInt => write!(f, "bigint"),
            Self::Function => write!(f, "function"),
            Self::Symbol => write!(f, "symbol"),
            Self::Undefined => write!(f, "undefined"),
            Self::Null => write!(f, "null"),
            Self::Object => write!(f, "object"),
            Self::Unknown => write!(f, "<unknown>"),
        }
    }
}

impl JsType {
    fn guess(js: &JsValue) -> JsType {
        if js.as_f64().is_some() {
            Self::Number
        } else if js.as_string().is_some() {
            Self::String
        } else if js.as_bool().is_some() {
            Self::Bool
        } else if js.is_array() {
            Self::Array
        } else if js.is_bigint() {
            Self::BigInt
        } else if js.is_function() {
            Self::Function
        } else if js.is_symbol() {
            Self::Symbol
        } else if js.is_undefined() {
            Self::Undefined
        } else if js.is_null() {
            Self::Null
        } else if js.is_object() {
            Self::Object
        } else {
            Self::Unknown
        }
    }
}

/// Convert WASM-compatible type involved in nRPC streaming to Rust-centric type
pub trait FromWasmStreamableType: Sized {
    /// Required method
    fn from_wasm_streamable(js: JsValue) -> Result<Self, WasmStreamableConversionError>;
}

macro_rules! trivial_convert_number {
    ($ty: ty) => {
        impl FromWasmStreamableType for $ty {
            fn from_wasm_streamable(js: JsValue) -> Result<Self, WasmStreamableConversionError> {
                if let Some(num) = js.as_f64() {
                    Ok(num as $ty)
                } else {
                    Err(WasmStreamableConversionError::UnexpectedType {
                        expected: JsType::Number,
                        got: JsType::guess(&js),
                    })
                }
            }
        }

        impl IntoWasmStreamableType for $ty {
            fn into_wasm_streamable(self) -> JsValue {
                self.into()
            }
        }
    };
}

trivial_convert_number! { f64 }
trivial_convert_number! { f32 }

trivial_convert_number! { isize }
trivial_convert_number! { usize }

trivial_convert_number! { i8 }
trivial_convert_number! { i16 }
trivial_convert_number! { i32 }
trivial_convert_number! { i64 }
trivial_convert_number! { i128 }

trivial_convert_number! { u8 }
trivial_convert_number! { u16 }
trivial_convert_number! { u32 }
trivial_convert_number! { u64 }
trivial_convert_number! { u128 }

impl FromWasmStreamableType for String {
    fn from_wasm_streamable(js: JsValue) -> Result<Self, WasmStreamableConversionError> {
        if let Some(s) = js.as_string() {
            Ok(s)
        } else {
            Err(WasmStreamableConversionError::UnexpectedType {
                expected: JsType::String,
                got: JsType::guess(&js),
            })
        }
    }
}

impl IntoWasmStreamableType for String {
    fn into_wasm_streamable(self) -> JsValue {
        self.into()
    }
}

impl FromWasmStreamableType for bool {
    fn from_wasm_streamable(js: JsValue) -> Result<Self, WasmStreamableConversionError> {
        if let Some(b) = js.as_bool() {
            Ok(b)
        } else {
            Err(WasmStreamableConversionError::UnexpectedType {
                expected: JsType::Bool,
                got: JsType::guess(&js),
            })
        }
    }
}

impl IntoWasmStreamableType for bool {
    fn into_wasm_streamable(self) -> JsValue {
        self.into()
    }
}

impl FromWasmStreamableType for () {
    fn from_wasm_streamable(_js: JsValue) -> Result<Self, WasmStreamableConversionError> {
        Ok(())
    }
}

impl IntoWasmStreamableType for () {
    fn into_wasm_streamable(self) -> JsValue {
        JsValue::undefined()
    }
}
