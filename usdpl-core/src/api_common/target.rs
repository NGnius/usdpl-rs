/// Supported plugin platforms
pub enum Platform {
    /// Generic platform
    Any,
    /// Decky aka PluginLoader platform
    Decky,
}

impl Platform {
    /// The current platform that usdpl-core is configured to target.
    /// This is determined by feature flags.
    pub fn current() -> Self {
        #[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
        {
            Self::Decky
        }
        #[cfg(not(any(feature = "decky", feature = "crankshaft")))]
        {
            Self::Any
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "any"),
            Self::Decky => write!(f, "decky"),
        }
    }
}
