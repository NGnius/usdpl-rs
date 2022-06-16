use base64::{decode_config_slice, encode_config_slice, Config};

const B64_CONF: Config = Config::new(base64::CharacterSet::Standard, true);

/// Errors from Loadable::load
#[derive(Debug)]
pub enum LoadError {
    TooSmallBuffer,
    InvalidData,
    #[cfg(debug_assertions)]
    Todo,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TooSmallBuffer => write!(f, "LoadError: TooSmallBuffer"),
            Self::InvalidData => write!(f, "LoadError: InvalidData"),
            #[cfg(debug_assertions)]
            Self::Todo => write!(f, "LoadError: TODO!"),
        }
    }
}

/// Load an object from the buffer
pub trait Loadable: Sized {
    /// Read the buffer, building the object and returning the amount of bytes read.
    /// If anything is wrong with the buffer, None should be returned.
    fn load(buffer: &[u8]) -> Result<(Self, usize), LoadError>;

    fn load_base64(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        let mut buffer2 = [0u8; crate::socket::PACKET_BUFFER_SIZE];
        let len = decode_config_slice(buffer, B64_CONF, &mut buffer2)
            .map_err(|_| LoadError::InvalidData)?;
        Self::load(&buffer2[..len])
    }
}

/// Errors from Dumpable::dump
#[derive(Debug)]
pub enum DumpError {
    TooSmallBuffer,
    Unsupported,
    #[cfg(debug_assertions)]
    Todo,
}

impl std::fmt::Display for DumpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TooSmallBuffer => write!(f, "DumpError: TooSmallBuffer"),
            Self::Unsupported => write!(f, "DumpError: Unsupported"),
            #[cfg(debug_assertions)]
            Self::Todo => write!(f, "DumpError: TODO!"),
        }
    }
}

/// Dump an object into the buffer
pub trait Dumpable {
    /// Write the object to the buffer, returning the amount of bytes written.
    /// If anything is wrong, false should be returned.
    fn dump(&self, buffer: &mut [u8]) -> Result<usize, DumpError>;

    fn dump_base64(&self, buffer: &mut [u8]) -> Result<usize, DumpError> {
        let mut buffer2 = [0u8; crate::socket::PACKET_BUFFER_SIZE];
        let len = self.dump(&mut buffer2)?;
        let len = encode_config_slice(&buffer2[..len], B64_CONF, buffer);
        Ok(len)
    }
}
