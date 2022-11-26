//! Directories that may be hard to determine when running from the plugin framework's environment

use std::path::{Path, PathBuf};
use std::process::Command;
//use std::io;

/// The home directory of the user currently running the Steam Deck UI (specifically: running gamescope).
pub fn home() -> Option<PathBuf> {
    let pid_out = Command::new("pidof")
                    .args(["gamescope"])
                    .output().ok()?;
    let pid_out_str = String::from_utf8_lossy(pid_out.stdout.as_slice());
    //println!("pidof gamescope: {}", pid_out_str);
    let pid_str = pid_out_str.split(" ").next()?.trim();
    let uid: String = super::files::read_single(format!("/proc/{}/loginuid", pid_str.trim())).ok()?;
    //println!("uid: {}", uid);
    //let pid: u32 = pid_str.parse().ok()?;
    let user_info = Command::new("bash")
                    .args(["-c", &format!("id {}", uid)])
                    .output().ok()?;
    let user_out_str = String::from_utf8_lossy(user_info.stdout.as_slice());
    //println!("loginuid: {}", user_out_str);
    let user_str = user_out_str.split(")").next()?;
    let user = &user_str[user_str.find("(")?+1..];
    Some(Path::new("/home").join(user))
}
