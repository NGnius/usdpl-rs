//! Datatypes and constants core the back-end and front-end libraries' operation.
//! This contains serialization functionality and networking datatypes.
mod remote_call;

pub mod socket;
pub mod serdes;

pub use remote_call::{RemoteCall, RemoteCallResponse};
