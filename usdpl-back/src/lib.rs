//! Back-end library for plugins.
//! Targets x86_64 (native Steam Deck ISA).
//!
//! This is a minimalist web server for handling events from the front-end.
//!
#![warn(missing_docs)]

#[cfg(not(any(feature = "decky")))]
mod api_any;
mod api_common;
#[cfg(all(feature = "decky", not(any(feature = "any"))))]
mod api_decky;

mod rpc;

//mod errors;
mod websockets;

pub use websockets::WebsocketServer as Server;
//pub use errors::{ServerError, ServerResult};

/// USDPL backend API.
/// This contains functionality used exclusively by the back-end.
pub mod api {
    pub use super::api_common::*;

    /// Standard interfaces not specific to a single plugin loader
    #[cfg(not(any(feature = "decky")))]
    pub mod any {
        pub use super::super::api_any::*;
    }

    /// Decky-specific interfaces
    #[cfg(all(feature = "decky", not(any(feature = "any"))))]
    pub mod decky {
        pub use super::super::api_decky::*;
    }
}

/// usdpl-core re-export
pub mod core {
    pub use usdpl_core::*;
}

/// nrpc re-export
pub mod nrpc {
    pub use nrpc::*;
}

/*/// nRPC-generated exports
#[allow(missing_docs)]
#[allow(dead_code)]
pub mod services {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}*/
