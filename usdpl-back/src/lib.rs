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
    pub use super::api_common::*;

    /// Standard interfaces not specific to a single plugin loader
    #[cfg(not(any(feature = "decky", feature = "crankshaft")))]
    pub mod any { pub use super::super::api_any::*; }

    /// Crankshaft-specific interfaces (FIXME)
    #[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
    pub mod crankshaft { pub use super::super::api_crankshaft::*; }

    /// Decky-specific interfaces
    #[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
    pub mod decky { pub use super::super::api_decky::*; }
}

/// usdpl-core re-export
pub mod core {
    pub use usdpl_core::*;
}
