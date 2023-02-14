//! Directories that may be hard to determine when running from the plugin framework's environment

use std::path::PathBuf;

/// The home directory of the user currently running the Steam Deck UI.
pub fn home() -> Option<PathBuf> {
    #[cfg(not(any(feature = "decky", feature = "crankshaft")))]
    let result = crate::api_any::dirs::home();
    #[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
    let result = None; // TODO
    #[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
    let result = crate::api_decky::home().ok().map(|x| x.into());

    result
}

/// The plugin's root folder.
pub fn plugin() -> Option<PathBuf> {
    #[cfg(not(any(feature = "decky", feature = "crankshaft")))]
    let result = None; // TODO
    #[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
    let result = None; // TODO
    #[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
    let result = crate::api_decky::plugin_dir().ok().map(|x| x.into());

    result
}

/// The recommended log directory
pub fn log() -> Option<PathBuf> {
    #[cfg(not(any(feature = "decky", feature = "crankshaft")))]
    let result = crate::api_any::dirs::log();
    #[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
    let result = None; // TODO
    #[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
    let result = crate::api_decky::log_dir().ok().map(|x| x.into());

    result
}
