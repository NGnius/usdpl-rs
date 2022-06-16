//! Back-end library for plugins.
//! Targets x86_64 (native Steam Deck ISA).
//!
//! This is a minimalist TCP server for handling events from the front-end.
//!

mod callable;
//mod errors;
mod instance;

pub use callable::Callable;
pub use instance::Instance;
//pub use errors::{ServerError, ServerResult};

pub mod core {
    pub use usdpl_core::*;
}
