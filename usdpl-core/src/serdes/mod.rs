//! Serialization and deserialization functionality.
//! Little endian is preferred.

mod dump_impl;
mod load_impl;
mod primitive;
mod traits;

pub use primitive::Primitive;
pub use traits::{DumpError, Dumpable, LoadError, Loadable};
