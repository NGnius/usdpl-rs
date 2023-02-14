use std::env::{self, VarError};

/// Decky version
pub fn version() -> Result<String, VarError> {
    env::var("DECKY_VERSION")
}

/// User running Decky
pub fn user() -> Result<String, VarError> {
    env::var("DECKY_USER")
}

/// Home of user running Decky
pub fn home() -> Result<String, VarError> {
    env::var("DECKY_HOME")
}

/// Settings directory recommended to be used by Decky
pub fn settings_dir() -> Result<String, VarError> {
    env::var("DECKY_PLUGIN_SETTINGS_DIR")
}

/// Runtime directory recommended to be used by Decky
pub fn runtime_dir() -> Result<String, VarError> {
    env::var("DECKY_PLUGIN_RUNTIME_DIR")
}

/// Log directory recommended to be used by Decky
pub fn log_dir() -> Result<String, VarError> {
    env::var("DECKY_PLUGIN_LOG_DIR")
}

/// Root directory of plugin
pub fn plugin_dir() -> Result<String, VarError> {
    env::var("DECKY_PLUGIN_DIR")
}

/// Plugin name
pub fn plugin_name() -> Result<String, VarError> {
    env::var("DECKY_PLUGIN_NAME")
}

/// Plugin version
pub fn plugin_version() -> Result<String, VarError> {
    env::var("DECKY_PLUGIN_VERSION")
}

/// Plugin author
pub fn plugin_author() -> Result<String, VarError> {
    env::var("DECKY_PLUGIN_AUTHOR")
}
