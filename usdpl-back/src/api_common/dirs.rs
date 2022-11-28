//! Directories that may be hard to determine when running from the plugin framework's environment

use std::path::{Path, PathBuf};
use std::process::Command;
//use std::io;

/// The home directory of the user currently running the Steam Deck UI (specifically: running gamescope).
pub fn home() -> Option<PathBuf> {
    let who_out = Command::new("who")
                    .output().ok()?;
    let who_str = String::from_utf8_lossy(who_out.stdout.as_slice());
    for login in who_str.split("\n") {
        let username = login
                        .split(" ")
                        .next()?
                        .trim();
        let path = Path::new("/home").join(username);
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn home_test() {
        let home_opt = home();
        assert!(home_opt.is_some(), "Expected valid home to be detected");
        let real_home = home_opt.unwrap();
        assert!(real_home.exists(), "Received invalid home dir");
        println!("Home dir detected as {}", real_home.display());
    }
}
