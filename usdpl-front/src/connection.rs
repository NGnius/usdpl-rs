use std::net::TcpStream;
use std::io::{Read, Write};

use web_sys::TcpSocket;
use js_sys::{ArrayBuffer, DataView};

use usdpl_core::socket;
use usdpl_core::serdes::{Dumpable, Loadable};

#[allow(dead_code)]
pub(crate) fn send(packet: socket::Packet) -> bool {
    let socket = match TcpSocket::new(socket::HOST_STR, socket::PORT) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
    let (ok, len) = packet.dump(&mut buffer);
    if !ok {
        return false;
    }
    // copy to JS buffer
    let array_buffer = ArrayBuffer::new(len as u32);
    let dataview = DataView::new(&array_buffer, 0, len);
    for i in 0..len {
        dataview.set_uint8(i, buffer[i]);
    }
    match socket.send_with_array_buffer(&array_buffer) {
        Ok(b) => b,
        Err(_) => false
    }
}

pub(crate) fn send_native(packet: socket::Packet) -> Option<socket::Packet> {
    let mut socket = match TcpStream::connect(socket::socket_addr()) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
    let (ok, len) = packet.dump(&mut buffer);
    if !ok {
        return None;
    }
    match socket.write(&buffer[..len]) {
        Ok(_) => {},
        Err(_) => return None
    }
    let len = match socket.read(&mut buffer) {
        Ok(len) => len,
        Err(_) => return None
    };
    socket::Packet::load(&buffer[..len]).0
}
