//! Back-end library for plugins.
//! Targets x86_64 (native Steam Deck ISA).
//!
//! This is a minimalist TCP server for handling events from the front-end.
//!

mod instance;

pub use instance::Instance;
