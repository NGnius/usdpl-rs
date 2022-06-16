//! Datatypes and constants core the back-end and front-end libraries' operation.
//! This contains serialization functionality and networking datatypes.
#![warn(missing_docs)]

mod remote_call;

#[cfg(not(any(feature = "decky", feature = "crankshaft")))]
mod api_any;
mod api_common;
#[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
mod api_crankshaft;
#[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
mod api_decky;

pub mod serdes;
pub mod socket;

pub use remote_call::{RemoteCall, RemoteCallResponse};

/// USDPL core API.
/// This contains functionality used in both the back-end and front-end.
pub mod api {
    #[cfg(not(any(feature = "decky", feature = "crankshaft")))]
    pub use super::api_any::*;
    pub use super::api_common::*;
    #[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
    pub use super::api_crankshaft::*;
    #[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
    pub use super::api_decky::*;
}
