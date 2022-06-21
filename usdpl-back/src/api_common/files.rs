//! Common low-level file operations
use std::fmt::Display;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write, self};
use std::str::FromStr;

/// Write something to a file.
/// Useful for kernel configuration files.
pub fn write_single<P: AsRef<Path>, D: Display>(path: P, display: D) -> Result<(), io::Error> {
    let mut file = File::create(path)?;
    write!(file, "{}\n", display)
}

/// Read something from a file.
/// Useful for kernel configuration files.
pub fn read_single<P: AsRef<Path>, D: FromStr<Err=E>, E>(path: P) -> Result<D, (Option<io::Error>, Option<E>)> {
    let mut file = File::create(path).map_err(|e| (Some(e), None))?;
    let mut string = String::new();
    file.read_to_string(&mut string).map_err(|e| (Some(e), None))?;
    string.parse().map_err(|e| (None, Some(e)))
}
