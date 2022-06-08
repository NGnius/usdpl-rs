use std::net::{SocketAddrV4, SocketAddr, Ipv4Addr};

use crate::serdes::{Loadable, Dumpable};
use crate::{RemoteCall, RemoteCallResponse};

pub const PORT: u16 = 31337;
pub const HTTP_PORT: u16 = 31338;
pub const HOST_STR: &str = "127.0.0.1";
pub const HOST: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
pub const SOCKET_ADDR_STR: &str = "127.0.0.1:31337";
//pub const SOCKET_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(HOST, PORT));

pub const PACKET_BUFFER_SIZE: usize = 1024;

pub fn socket_addr() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(HOST, PORT))
}

pub enum Packet {
    Call(RemoteCall),
    CallResponse(RemoteCallResponse),
    KeepAlive,
    Invalid,
    Message(String),
    Unsupported,
    Bad,
}

impl Packet {
    const fn discriminant(&self) -> u8 {
        match self {
            Self::Call(_) => 1,
            Self::CallResponse(_) => 2,
            Self::KeepAlive => 3,
            Self::Invalid => 4,
            Self::Message(_) => 5,
            Self::Unsupported => 6,
            Self::Bad => 7,
        }
    }
}

impl Loadable for Packet {
    fn load(buf: &[u8]) -> (Option<Self>, usize) {
        if buf.len() == 0 {
            return (None, 1);
        }
        let mut result: (Option<Self>, usize) = match buf[0] {
            //0 => (None, 0),
            1 => {
                let (obj, len) = RemoteCall::load(&buf[1..]);
                (obj.map(Self::Call), len)
            },
            2 => {
                let (obj, len) = RemoteCallResponse::load(&buf[1..]);
                (obj.map(Self::CallResponse), len)
            },
            3 => (Some(Self::KeepAlive), 0),
            4 => (Some(Self::Invalid), 0),
            5 => {
                let (obj, len) = String::load(&buf[1..]);
                (obj.map(Self::Message), len)
            },
            6 => (Some(Self::Unsupported), 0),
            7 => (None, 0),
            _ => (None, 0)
        };
        result.1 += 1;
        result
    }
}

impl Dumpable for Packet {
    fn dump(&self, buf: &mut [u8]) -> (bool, usize) {
        if buf.len() == 0 {
            return (false, 0);
        }
        buf[0] = self.discriminant();
        let mut result = match self {
            Self::Call(c) => c.dump(&mut buf[1..]),
            Self::CallResponse(c) => c.dump(&mut buf[1..]),
            Self::KeepAlive => (true, 0),
            Self::Invalid => (true, 0),
            Self::Message(s) => s.dump(&mut buf[1..]),
            Self::Unsupported => (true, 0),
            Self::Bad => (false, 0),
        };
        result.1 += 1;
        result
    }
}
