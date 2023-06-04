/// A Rust type which supports Into/FromWasmAbi or WasmDescribe
pub trait KnownWasmCompatible {}

/// Convert Rust type to WASM-compatible type
pub trait IntoWasmable<T: KnownWasmCompatible> {
    /// Required method
    fn into_wasm(self) -> T;
}

/// Convert WASM-compatible type to Rust-centric type
pub trait FromWasmable<T: KnownWasmCompatible> {
    /// Required method
    fn from_wasm(js: T) -> Self;
}

impl KnownWasmCompatible for f64 {}
impl KnownWasmCompatible for f32 {}

impl KnownWasmCompatible for isize {}
impl KnownWasmCompatible for usize {}

impl KnownWasmCompatible for i8 {}
impl KnownWasmCompatible for i16 {}
impl KnownWasmCompatible for i32 {}
impl KnownWasmCompatible for i64 {}
impl KnownWasmCompatible for i128 {}

impl KnownWasmCompatible for u8 {}
impl KnownWasmCompatible for u16 {}
impl KnownWasmCompatible for u32 {}
impl KnownWasmCompatible for u64 {}
impl KnownWasmCompatible for u128 {}

impl KnownWasmCompatible for bool {}
impl KnownWasmCompatible for String {}

impl KnownWasmCompatible for () {}

impl KnownWasmCompatible for js_sys::Map {}
impl KnownWasmCompatible for js_sys::Array {}
