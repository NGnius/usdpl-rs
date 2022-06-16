//! Web messaging
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::serdes::{DumpError, Dumpable, LoadError, Loadable};
use crate::{RemoteCall, RemoteCallResponse};

/// Host IP address for web browsers
pub const HOST_STR: &str = "localhost";
/// Host IP address
pub const HOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

/// Standard max packet size
pub const PACKET_BUFFER_SIZE: usize = 1024;

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
        }
    }
}

impl Loadable for Packet {
    fn load(buf: &[u8]) -> Result<(Self, usize), LoadError> {
        if buf.len() == 0 {
            return Err(LoadError::TooSmallBuffer);
        }
        let mut result: (Self, usize) = match buf[0] {
            //0 => (None, 0),
            1 => {
                let (obj, len) = RemoteCall::load(&buf[1..])?;
                (Self::Call(obj), len)
            }
            2 => {
                let (obj, len) = RemoteCallResponse::load(&buf[1..])?;
                (Self::CallResponse(obj), len)
            }
            3 => (Self::KeepAlive, 0),
            4 => (Self::Invalid, 0),
            5 => {
                let (obj, len) = String::load(&buf[1..])?;
                (Self::Message(obj), len)
            }
            6 => (Self::Unsupported, 0),
            7 => return Err(LoadError::InvalidData),
            8 => {
                let (obj, len) = <_>::load(&buf[1..])?;
                (Self::Many(obj), len)
            }
            _ => return Err(LoadError::InvalidData),
        };
        result.1 += 1;
        Ok(result)
    }
}

impl Dumpable for Packet {
    fn dump(&self, buf: &mut [u8]) -> Result<usize, DumpError> {
        if buf.len() == 0 {
            return Err(DumpError::TooSmallBuffer);
        }
        buf[0] = self.discriminant();
        let mut result = match self {
            Self::Call(c) => c.dump(&mut buf[1..]),
            Self::CallResponse(c) => c.dump(&mut buf[1..]),
            Self::KeepAlive => Ok(0),
            Self::Invalid => Ok(0),
            Self::Message(s) => s.dump(&mut buf[1..]),
            Self::Unsupported => Ok(0),
            Self::Bad => return Err(DumpError::Unsupported),
            Self::Many(v) => v.dump(&mut buf[1..]),
        }?;
        result += 1;
        Ok(result)
    }
}
