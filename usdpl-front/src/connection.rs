use std::net::TcpStream;
use std::io::{Read, Write};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{WebSocket, MessageEvent, ErrorEvent};
use js_sys::{ArrayBuffer, DataView, Uint8Array};
use wasm_rs_shared_channel::{Expects, spsc::{Receiver, Sender}};

use usdpl_core::socket;
use usdpl_core::serdes::{Dumpable, Loadable};

use super::imports;

#[allow(dead_code)]
/// Send packet over a Javascript socket
pub(crate) fn send_js(packet: socket::Packet, port: u16) -> Option<socket::Packet> {
    let addr = format!("wss://{}:{}", socket::HOST_STR, port);
    let socket = match WebSocket::new(&addr) {
        Ok(s) => s,
        Err(e) => {
            imports::console_error(
                &format!("USDPL error: TcpSocket::new(...) failed with error {}",
                js_sys::JSON::stringify(&e)
                    .map(|x| x.as_string().unwrap_or("WTF".into()))
                    .unwrap_or("unknown error".into())));
            return None;
        }
    };
    socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
    let (tx, rx) : (Sender<_>, Receiver<_>) = wasm_rs_shared_channel::spsc::channel(socket::PACKET_BUFFER_SIZE as u32 + 4).split();

    let onmessage_callback = Closure::wrap(Box::new(onmessage_factory(tx)) as Box<dyn FnMut(MessageEvent)>);
    socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    //onmessage_callback.forget();

    let onerror_callback = Closure::wrap(Box::new(onerror_factory()) as Box<dyn FnMut(ErrorEvent)>);
    socket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    //onerror_callback.forget();

    let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
    let (ok, len) = packet.dump(&mut buffer);
    if !ok {
        imports::console_error("USDPL error: packet dump failed");
        return None;
    }
    // copy to JS buffer
    let array_buffer = ArrayBuffer::new(len as u32);
    let dataview = DataView::new(&array_buffer, 0, len);
    for i in 0..len {
        dataview.set_uint8(i, buffer[i]);
    }
    match socket.send_with_array_buffer(&array_buffer) {
        Ok(_) => {},
        Err(e) => {
            imports::console_error(&format!("USDPL error: socket send_with_array_buffer(...) failed -- {:?}", e));
            return None;
        }
    }
    let result = match rx.recv(Some(std::time::Duration::from_secs(60))) {
        Ok(Some(val)) => {
            socket::Packet::load(&val.1[..val.0 as _]).0
        },
        Ok(None) => {
            imports::console_error(&format!("USDPL error: SharedChannel recv timed out"));
            None
        },
        Err(e) => {
            imports::console_error(&format!("USDPL error: got SharedChannel recv error -- {:?}", e));
            None
        }
    };
    socket.close().unwrap_or(());
    result
}

fn onmessage_factory(sender: Sender<Sendable<{socket::PACKET_BUFFER_SIZE}>>) -> impl FnMut(MessageEvent) {
    move |e: MessageEvent| {
        if let Ok(buf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
            let dataview = DataView::new(&buf, 0, buf.byte_length() as _);
            for i in 0..buf.byte_length() as usize {
                if i < socket::PACKET_BUFFER_SIZE {
                    buffer[i] = dataview.get_uint8(i);
                } else {
                    break;
                }
            }
            if let Err(e) = sender.send(&Sendable(buf.byte_length(), buffer)) {
                imports::console_error(&format!("USDPL error: got SharedChannel send error {:?}", e));
            }
        } else {
            imports::console_warn(&format!("USDPL warning: Got non-data message from {}", e.origin()));
        }
    }
}

fn onerror_factory() -> impl FnMut(ErrorEvent) {
    move |e: ErrorEvent| {
        imports::console_error(&format!("USDPL error: got socket error {}", e.message()))
    }
}

#[allow(dead_code)]
/// Send packet over a WASM-native TCP socket
pub(crate) fn send_native(packet: socket::Packet, port: u16) -> Option<socket::Packet> {
    let mut socket = match TcpStream::connect(socket::socket_addr(port)) {
        Ok(s) => s,
        Err(e) => {
            imports::console_error(&format!("USDPL error: TcpStream failed to connect with error {}", e));
            return None;
        },
    };
    let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
    let (ok, len) = packet.dump(&mut buffer);
    if !ok {
        imports::console_error("USDPL error: packet dump failed");
        return None;
    }
    match socket.write(&buffer[..len]) {
        Ok(_) => {},
        Err(e) => {
            imports::console_error(&format!("USDPL error: socket write failed with error {}", e));
            return None;
        }
    }
    let len = match socket.read(&mut buffer) {
        Ok(len) => len,
        Err(e) => {
            imports::console_error(&format!("USDPL error: socket read failed with error {}", e));
            return None;
        }
    };
    socket::Packet::load(&buffer[..len]).0
}

struct Sendable<const SIZE: usize>(u32, [u8; SIZE]);

#[derive(Debug)]
struct SendableError(String);

impl std::error::Error for SendableError {}

impl std::fmt::Display for SendableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&self.0 as &dyn std::fmt::Display).fmt(f)
    }
}

impl<const SIZE: usize> wasm_rs_shared_channel::Shareable for Sendable<SIZE> {
    type Error = SendableError;

    fn to_bytes(&self) -> Result<Uint8Array, Self::Error> {
        let array = Uint8Array::new_with_length(SIZE as u32 + 4);
        let mut cursor = 0;
        for byte in self.0.to_le_bytes() {
            array.set_index(cursor, byte);
            cursor += 1;
        }
        for byte in self.1 {
            array.set_index(cursor, byte);
            cursor += 1;
        }
        Ok(array)
    }

    fn from(bytes: &Uint8Array) -> Result<Result<Self, Expects>, Self::Error> {
        if bytes.length() < 4 {
            return Err(SendableError("Too small for size int".into()));
        }
        let len = u32::from_le_bytes([
            bytes.get_index(0),
            bytes.get_index(1),
            bytes.get_index(2),
            bytes.get_index(3),
        ]);
        if bytes.length() < len + 4 {
            return Err(SendableError("Too small for buffer".into()));
        }
        let mut buf = [0u8; SIZE];
        for i in 0..len {
            buf[i as usize] = bytes.get_index(4 + i);
        }
        Ok(Ok(Sendable(len, buf)))
    }
}
