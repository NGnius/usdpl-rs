//! WASM <-> Rust interop utilities
mod arrays;
mod js_function_stream;
mod maps;
mod streaming;
mod trivials;
mod wasm_traits;

pub use js_function_stream::JsFunctionStream;
pub use wasm_traits::*;
pub use streaming::*;
