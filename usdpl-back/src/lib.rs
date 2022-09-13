//! Back-end library for plugins.
//! Targets x86_64 (native Steam Deck ISA).
//!
//! This is a minimalist web server for handling events from the front-end.
//!
#![warn(missing_docs)]

#[cfg(not(any(feature = "decky", feature = "crankshaft")))]
mod api_any;
mod api_common;
#[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
mod api_crankshaft;
#[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
mod api_decky;

mod callable;
//mod errors;
mod instance;

pub use callable::{Callable, MutCallable, AsyncCallable};
pub(crate) use callable::WrappedCallable;
pub use instance::Instance;
//pub use errors::{ServerError, ServerResult};

/// USDPL backend API.
/// This contains functionality used exclusively by the back-end.
pub mod api {
    #[cfg(not(any(feature = "decky", feature = "crankshaft")))]
    pub use super::api_any::*;
    pub use super::api_common::*;
    #[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
    pub use super::api_crankshaft::*;
    #[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
    pub use super::api_decky::*;
}

/// usdpl-core re-export
pub mod core {
    pub use usdpl_core::*;
}
