use base64::{decode_config_slice, encode_config_slice, Config};

const B64_CONF: Config = Config::new(base64::CharacterSet::Standard, true);

#[cfg(feature = "encrypt")]
const ASSOCIATED_DATA: &[u8] = b"usdpl-core-data";

#[cfg(feature = "encrypt")]
use aes_gcm_siv::aead::{AeadInPlace, NewAead};

/// Errors from Loadable::load
#[derive(Debug)]
pub enum LoadError {
    /// Buffer smaller than expected
    TooSmallBuffer,
    /// Unexpected/corrupted data encountered
    InvalidData,
    /// Encrypted data cannot be decrypted
    #[cfg(feature = "encrypt")]
    DecryptionError,
    /// Unimplemented
    #[cfg(debug_assertions)]
    Todo,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TooSmallBuffer => write!(f, "LoadError: TooSmallBuffer"),
            Self::InvalidData => write!(f, "LoadError: InvalidData"),
            #[cfg(feature = "encrypt")]
            Self::DecryptionError => write!(f, "LoadError: DecryptionError"),
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

    /// Load data from a base64-encoded buffer
    fn load_base64(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        let mut buffer2 = [0u8; crate::socket::PACKET_BUFFER_SIZE];
        let len = decode_config_slice(buffer, B64_CONF, &mut buffer2)
            .map_err(|_| LoadError::InvalidData)?;
        Self::load(&buffer2[..len])
    }

    /// Load data from an encrypted base64-encoded buffer
    #[cfg(feature = "encrypt")]
    fn load_encrypted(buffer: &[u8], key: &[u8], nonce: &[u8]) -> Result<(Self, usize), LoadError> {
        println!("encrypted buffer: {}", String::from_utf8(buffer.to_vec()).unwrap());
        let key = aes_gcm_siv::Key::from_slice(key);
        let cipher = aes_gcm_siv::Aes256GcmSiv::new(key);
        let nonce = aes_gcm_siv::Nonce::from_slice(nonce);
        let mut decoded_buf = base64::decode_config(buffer, B64_CONF)
            .map_err(|_| LoadError::InvalidData)?;
        println!("Decoded buf: {:?}", decoded_buf);
        cipher.decrypt_in_place(nonce, ASSOCIATED_DATA, &mut decoded_buf).map_err(|_| LoadError::DecryptionError)?;
        println!("Decrypted buf: {:?}", decoded_buf);
        Self::load(decoded_buf.as_slice())
    }
}

/// Errors from Dumpable::dump
#[derive(Debug)]
pub enum DumpError {
    /// Buffer not big enough to dump data into
    TooSmallBuffer,
    /// Data cannot be dumped
    Unsupported,
    /// Data cannot be encrypted
    #[cfg(feature = "encrypt")]
    EncryptionError,
    /// Unimplemented
    #[cfg(debug_assertions)]
    Todo,
}

impl std::fmt::Display for DumpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TooSmallBuffer => write!(f, "DumpError: TooSmallBuffer"),
            Self::Unsupported => write!(f, "DumpError: Unsupported"),
            #[cfg(feature = "encrypt")]
            Self::EncryptionError => write!(f, "DumpError: EncryptionError"),
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

    /// Dump data as base64-encoded.
    /// Useful for transmitting data as text.
    fn dump_base64(&self, buffer: &mut [u8]) -> Result<usize, DumpError> {
        let mut buffer2 = [0u8; crate::socket::PACKET_BUFFER_SIZE];
        let len = self.dump(&mut buffer2)?;
        let len = encode_config_slice(&buffer2[..len], B64_CONF, buffer);
        Ok(len)
    }

    /// Dump data as an encrypted base64-encoded buffer
    #[cfg(feature = "encrypt")]
    fn dump_encrypted(&self, buffer: &mut Vec<u8>, key: &[u8], nonce: &[u8]) -> Result<usize, DumpError> {
        let mut buffer2 = Vec::with_capacity(buffer.capacity());
        buffer2.extend_from_slice(buffer.as_slice());
        let size = self.dump(&mut buffer2)?;
        buffer2.truncate(size);
        println!("Buf: {:?}", buffer2);
        let key = aes_gcm_siv::Key::from_slice(key);
        let cipher = aes_gcm_siv::Aes256GcmSiv::new(key);
        let nonce = aes_gcm_siv::Nonce::from_slice(nonce);
        cipher.encrypt_in_place(nonce, ASSOCIATED_DATA, &mut buffer2).map_err(|_| DumpError::EncryptionError)?;
        println!("Encrypted slice: {:?}", &buffer2);
        let size = encode_config_slice(buffer2.as_slice(), B64_CONF, buffer);
        let string = String::from_utf8(buffer.as_slice()[..size].to_vec()).unwrap();
        println!("Encoded slice: {}", string);
        Ok(size)
    }
}
