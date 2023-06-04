use super::{FromWasmable, IntoWasmable};

macro_rules! trivial_convert {
    ($ty: ty) => {
        impl FromWasmable<$ty> for $ty {
            fn from_wasm(js: $ty) -> Self {
                js
            }
        }

        impl IntoWasmable<$ty> for $ty {
            fn into_wasm(self) -> $ty {
                self
            }
        }
    };
}

trivial_convert! { f64 }
trivial_convert! { f32 }

trivial_convert! { isize }
trivial_convert! { usize }

trivial_convert! { i8 }
trivial_convert! { i16 }
trivial_convert! { i32 }
trivial_convert! { i64 }
trivial_convert! { i128 }

trivial_convert! { u8 }
trivial_convert! { u16 }
trivial_convert! { u32 }
trivial_convert! { u64 }
trivial_convert! { u128 }

trivial_convert! { bool }
trivial_convert! { String }

trivial_convert! { () }
