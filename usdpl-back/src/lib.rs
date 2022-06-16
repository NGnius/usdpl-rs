//! Back-end library for plugins.
//! Targets x86_64 (native Steam Deck ISA).
//!
//! This is a minimalist web server for handling events from the front-end.
//!
#![warn(missing_docs)]

mod callable;
//mod errors;
mod instance;

pub use callable::Callable;
pub use instance::Instance;
//pub use errors::{ServerError, ServerResult};

/// usdpl-core re-export
pub mod core {
    pub use usdpl_core::*;
}
