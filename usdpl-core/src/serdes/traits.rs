use std::io::{Read, Write, Cursor};
use base64::{decode_config_buf, encode_config_buf, Config};

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
    /// Read error
    Io(std::io::Error),
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
            Self::Io(err) => write!(f, "LoadError: Io({})", err),
            #[cfg(debug_assertions)]
            Self::Todo => write!(f, "LoadError: TODO!"),
        }
    }
}

/// Load an object from the buffer
pub trait Loadable: Sized {
    /// Read the buffer, building the object and returning the amount of bytes read.
    /// If anything is wrong with the buffer, Err should be returned.
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError>;

    /// Load data from a base64-encoded buffer
    fn load_base64(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        let mut buffer2 = Vec::with_capacity(crate::socket::PACKET_BUFFER_SIZE);
        decode_config_buf(buffer, B64_CONF, &mut buffer2)
            .map_err(|_| LoadError::InvalidData)?;
        let mut cursor = Cursor::new(buffer2);
        Self::load(&mut cursor)
    }

    /// Load data from an encrypted base64-encoded buffer
    #[cfg(feature = "encrypt")]
    fn load_encrypted(buffer: &[u8], key: &[u8], nonce: &[u8]) -> Result<(Self, usize), LoadError> {
        //println!("encrypted buffer: {}", String::from_utf8(buffer.to_vec()).unwrap());
        let key = aes_gcm_siv::Key::from_slice(key);
        let cipher = aes_gcm_siv::Aes256GcmSiv::new(key);
        let nonce = aes_gcm_siv::Nonce::from_slice(nonce);
        let mut decoded_buf = Vec::with_capacity(crate::socket::PACKET_BUFFER_SIZE);
        base64::decode_config_buf(buffer, B64_CONF, &mut decoded_buf)
            .map_err(|_| LoadError::InvalidData)?;
        //println!("Decoded buf: {:?}", decoded_buf);
        cipher.decrypt_in_place(nonce, ASSOCIATED_DATA, &mut decoded_buf).map_err(|_| LoadError::DecryptionError)?;
        //println!("Decrypted buf: {:?}", decoded_buf);
        let mut cursor = Cursor::new(decoded_buf);
        Self::load(&mut cursor)
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
    /// Write error
    Io(std::io::Error),
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
            Self::Io(err) => write!(f, "DumpError: Io({})", err),
            #[cfg(debug_assertions)]
            Self::Todo => write!(f, "DumpError: TODO!"),
        }
    }
}

/// Dump an object into the buffer
pub trait Dumpable {
    /// Write the object to the buffer, returning the amount of bytes written.
    /// If anything is wrong, false should be returned.
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError>;

    /// Dump data as base64-encoded.
    /// Useful for transmitting data as text.
    fn dump_base64(&self, buffer: &mut String) -> Result<usize, DumpError> {
        let mut buffer2 = Vec::with_capacity(crate::socket::PACKET_BUFFER_SIZE);
        let len = self.dump(&mut buffer2)?;
        encode_config_buf(&buffer2[..len], B64_CONF, buffer);
        Ok(len)
    }

    /// Dump data as an encrypted base64-encoded buffer
    #[cfg(feature = "encrypt")]
    fn dump_encrypted(&self, buffer: &mut Vec<u8>, key: &[u8], nonce: &[u8]) -> Result<usize, DumpError> {
        let mut buffer2 = Vec::with_capacity(crate::socket::PACKET_BUFFER_SIZE);
        let size = self.dump(&mut buffer2)?;
        buffer2.truncate(size);
        //println!("Buf: {:?}", buffer2);
        let key = aes_gcm_siv::Key::from_slice(key);
        let cipher = aes_gcm_siv::Aes256GcmSiv::new(key);
        let nonce = aes_gcm_siv::Nonce::from_slice(nonce);
        cipher.encrypt_in_place(nonce, ASSOCIATED_DATA, &mut buffer2).map_err(|_| DumpError::EncryptionError)?;
        //println!("Encrypted slice: {:?}", &buffer2);
        let mut base64_buf = String::with_capacity(crate::socket::PACKET_BUFFER_SIZE);
        encode_config_buf(buffer2.as_slice(), B64_CONF, &mut base64_buf);
        //println!("base64 len: {}", base64_buf.as_bytes().len());
        buffer.extend_from_slice(base64_buf.as_bytes());
        //let string = String::from_utf8(buffer.as_slice().to_vec()).unwrap();
        //println!("Encoded slice: {}", string);
        Ok(base64_buf.len())
    }
}
