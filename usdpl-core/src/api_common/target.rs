pub enum Platform {
    Any,
    Decky,
    Crankshaft,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(all(feature = "decky", not(any(feature = "crankshaft"))))]
        {
            Self::Decky
        }
        #[cfg(all(feature = "crankshaft", not(any(feature = "decky"))))]
        {
            Self::Crankshaft
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
            Self::Crankshaft => write!(f, "crankshaft"),
        }
    }
}
