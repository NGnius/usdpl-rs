//! Web messaging
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::io::{Read, Write};

use crate::serdes::{DumpError, Dumpable, LoadError, Loadable};
use crate::{RemoteCall, RemoteCallResponse};

/// Host IP address for web browsers
pub const HOST_STR: &str = "localhost";
/// Host IP address
pub const HOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

/// Standard max packet size
pub const PACKET_BUFFER_SIZE: usize = 1024;
/// Encryption nonce size
pub const NONCE_SIZE: usize = 12;

/// Address and port
#[inline]
pub fn socket_addr(port: u16) -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(HOST, port))
}

/// Accepted Packet types and the data they contain
pub enum Packet {
    /// A remote call
    Call(RemoteCall),
    /// A reponse to a remote call
    CallResponse(RemoteCallResponse),
    /// Unused
    KeepAlive,
    /// Invalid
    Invalid,
    /// General message
    Message(String),
    /// Response to an unsupported packet
    Unsupported,
    /// Broken packet type, useful for testing
    Bad,
    /// Many packets merged into one
    Many(Vec<Packet>),
    /// Translation data dump
    Translations(Vec<(String, Vec<String>)>),
    /// Request translations for language
    Language(String),
}

impl Packet {
    /// Byte representing the packet type -- the first byte of any packet in USDPL
    const fn discriminant(&self) -> u8 {
        match self {
            Self::Call(_) => 1,
            Self::CallResponse(_) => 2,
            Self::KeepAlive => 3,
            Self::Invalid => 4,
            Self::Message(_) => 5,
            Self::Unsupported => 6,
            Self::Bad => 7,
            Self::Many(_) => 8,
            Self::Translations(_) => 9,
            Self::Language(_) => 10,
        }
    }
}

impl Loadable for Packet {
    fn load(buf: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let mut discriminant_buf = [u8::MAX; 1];
        buf.read_exact(&mut discriminant_buf).map_err(LoadError::Io)?;
        let mut result: (Self, usize) = match discriminant_buf[0] {
            //0 => (None, 0),
            1 => {
                let (obj, len) = RemoteCall::load(buf)?;
                (Self::Call(obj), len)
            }
            2 => {
                let (obj, len) = RemoteCallResponse::load(buf)?;
                (Self::CallResponse(obj), len)
            }
            3 => (Self::KeepAlive, 0),
            4 => (Self::Invalid, 0),
            5 => {
                let (obj, len) = String::load(buf)?;
                (Self::Message(obj), len)
            }
            6 => (Self::Unsupported, 0),
            7 => return Err(LoadError::InvalidData),
            8 => {
                let (obj, len) = <_>::load(buf)?;
                (Self::Many(obj), len)
            },
            9 => {
                let (obj, len) = <_>::load(buf)?;
                (Self::Translations(obj), len)
            },
            10 => {
                let (obj, len) = <_>::load(buf)?;
                (Self::Language(obj), len)
            },
            _ => return Err(LoadError::InvalidData),
        };
        result.1 += 1;
        Ok(result)
    }
}

impl Dumpable for Packet {
    fn dump(&self, buf: &mut dyn Write) -> Result<usize, DumpError> {
        let size1 = buf.write(&[self.discriminant()]).map_err(DumpError::Io)?;
        let result = match self {
            Self::Call(c) => c.dump(buf),
            Self::CallResponse(c) => c.dump(buf),
            Self::KeepAlive => Ok(0),
            Self::Invalid => Ok(0),
            Self::Message(s) => s.dump(buf),
            Self::Unsupported => Ok(0),
            Self::Bad => return Err(DumpError::Unsupported),
            Self::Many(v) => v.dump(buf),
            Self::Translations(tr) => tr.dump(buf),
            Self::Language(l) => l.dump(buf),
        }?;
        Ok(size1 + result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "encrypt")]
    #[test]
    fn encryption_integration_test() {
        let key = hex_literal::hex!("59C4E408F27250B3147E7724511824F1D28ED7BEF43CF7103ACE747F77A2B265");
        let nonce = [0u8; NONCE_SIZE];
        let packet = Packet::Call(RemoteCall{
            id: 42,
            function: "test".into(),
            parameters: Vec::new(),
        });
        let mut buffer = Vec::with_capacity(PACKET_BUFFER_SIZE);
        let len = packet.dump_encrypted(&mut buffer, &key, &nonce).unwrap();
        println!("buffer: {}", String::from_utf8(buffer.as_slice()[..len].to_vec()).unwrap());

        let (packet_out, _len) = Packet::load_encrypted(&buffer.as_slice()[..len], &key, &nonce).unwrap();

        if let Packet::Call(call_out) = packet_out {
            if let Packet::Call(call_in) = packet {
                assert_eq!(call_in.id, call_out.id, "Input and output packets do not match");
                assert_eq!(call_in.function, call_out.function, "Input and output packets do not match");
                assert_eq!(call_in.parameters.len(), call_out.parameters.len(), "Input and output packets do not match");
            } else {
                panic!("Packet in not a Call");
            }
        } else {
            panic!("Packet out not a Call!");
        }
    }
}
