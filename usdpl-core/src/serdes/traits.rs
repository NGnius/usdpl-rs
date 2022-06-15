use base64::{encode_config_slice, decode_config_slice, Config};

const B64_CONF: Config = Config::new(base64::CharacterSet::Standard, true);

/// Load an object from the buffer
pub trait Loadable: Sized {
    /// Read the buffer, building the object and returning the amount of bytes read.
    /// If anything is wrong with the buffer, None should be returned.
    fn load(buffer: &[u8]) -> (Option<Self>, usize);

    fn load_base64(buffer: &[u8]) -> (Option<Self>, usize) {
        let mut buffer2 = [0u8; crate::socket::PACKET_BUFFER_SIZE];
        let len = match decode_config_slice(buffer, B64_CONF, &mut buffer2) {
            Ok(len) => len,
            Err(_) => return (None, 0),
        };
        Self::load(&buffer2[..len])
    }
}

/// Dump an object into the buffer
pub trait Dumpable {
    /// Write the object to the buffer, returning the amount of bytes written.
    /// If anything is wrong, false should be returned.
    fn dump(&self, buffer: &mut [u8]) -> (bool, usize);

    fn dump_base64(&self, buffer: &mut [u8]) -> (bool, usize) {
        let mut buffer2 = [0u8; crate::socket::PACKET_BUFFER_SIZE];
        let (ok, len) = self.dump(&mut buffer2);
        if !ok {
            return (false, len)
        }
        let len = encode_config_slice(&buffer2[..len], B64_CONF, buffer);
        (true, len)
    }
}
