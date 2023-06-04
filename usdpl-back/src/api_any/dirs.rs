//! Directories that may be hard to determine when running from the plugin framework's environment

use std::path::{Path, PathBuf};
use std::process::Command;

/// The home directory of the user currently running the Steam Deck UI (specifically: running gamescope).
pub fn home() -> Option<PathBuf> {
    let who_out = Command::new("who").output().ok()?;
    let who_str = String::from_utf8_lossy(who_out.stdout.as_slice());
    for login in who_str.split("\n") {
        let username = login.split(" ").next()?.trim();
        let path = Path::new("/home").join(username);
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

/// The recommended log directory
pub fn log() -> Option<PathBuf> {
    Some(PathBuf::from("/tmp"))
}
