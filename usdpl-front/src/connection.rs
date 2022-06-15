//use std::net::TcpStream;
//use std::io::{Read, Write};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

//use web_sys::{WebSocket, MessageEvent, ErrorEvent};
use web_sys::{Request, RequestInit, RequestMode, Response};
use js_sys::{ArrayBuffer, DataView, Uint8Array, JsString};
//use wasm_rs_shared_channel::{Expects, spsc::{Receiver, Sender}};

use usdpl_core::socket;
use usdpl_core::serdes::{Dumpable, Loadable, Primitive};

use super::imports;

pub async fn send_js(packet: socket::Packet, port: u16) -> Result<Vec<Primitive>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let url = format!("http://localhost:{}/usdpl/call", port);

    let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
    let (ok, len) = packet.dump_base64(&mut buffer);
    if !ok {
        imports::console_error("USDPL error: packet dump failed");
        return Err("Packet dump failed".into());
    }
    let string: String = String::from_utf8_lossy(&buffer[..len]).into();
    opts.body(Some(&string.into()));

    let request = Request::new_with_str_and_init(&url, &opts)?;

    request
        .headers()
        .set("Accept", "application/bytes")?;
        //.set("Authorization", "wasm TODO_KEY")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;

    // Convert this other `Promise` into a rust `Future`.
    let text = JsFuture::from(resp.text()?).await?;

    let string: JsString = text.dyn_into()?;

    match socket::Packet::load_base64(&string.as_string().unwrap().as_bytes()).0 {
        Some(socket::Packet::CallResponse(resp)) => {
            Ok(resp.response)
        },
        _ => {
            imports::console_warn(&format!("USDPL warning: Got non-call-response message from {}", resp.url()));
            Err("".into())
        }
    }
}

/*#[allow(dead_code)]
/// Send packet over a Javascript socket
pub(crate) fn send_js(packet: socket::Packet, port: u16, callback: js_sys::Function) -> bool {
    let addr = format!("wss://{}:{}",
        "192.168.0.128",//socket::HOST_STR,
        port);

    let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
    let (ok, len) = packet.dump(&mut buffer);
    if !ok {
        imports::console_error("USDPL error: packet dump failed");
        return false;
    }
    // copy to JS buffer
    let array_buffer = ArrayBuffer::new(len as _);
    let dataview = DataView::new(&array_buffer, 0, len);
    for i in 0..len {
        dataview.set_uint8(i, buffer[i]);
    }

    imports::console_log("USDPL: creating WebSocket");

    let socket = match WebSocket::new("wss://demo.piesocket.com/v3/channel_1?api_key=VCXCEuvhGcBDP7XhiJJUDvR1e1D3eiVjgZ9VRiaV&notify_self"/*addr*/) {
        Ok(s) => s,
        Err(e) => {
            imports::console_error(
                &format!("USDPL error: TcpSocket::new(...) failed with error {}",
                js_sys::JSON::stringify(&e)
                    .map(|x| x.as_string().unwrap_or("WTF".into()))
                    .unwrap_or("unknown error".into())));
            return false;
        }
    };
    socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
    //let (tx, rx) : (Sender<_>, Receiver<_>) = wasm_rs_shared_channel::spsc::channel(socket::PACKET_BUFFER_SIZE as u32 + 4).split();

    let onmessage_callback = Closure::wrap(Box::new(onmessage_factory(callback)) as Box<dyn FnMut(MessageEvent)>);
    //socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let onerror_callback = Closure::wrap(Box::new(onerror_factory()) as Box<dyn FnMut(ErrorEvent)>);
    socket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let onopen_callback = Closure::wrap(Box::new(onopen_factory(array_buffer, socket.clone())) as Box<dyn FnMut(JsValue)>);
    socket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    imports::console_log("USDPL: socket initialized");
    true
}

fn onmessage_factory(callback: js_sys::Function) -> impl FnMut(MessageEvent) {
    move |e: MessageEvent| {
        super::imports::console_log("USDPL: Got message");
        if let Ok(buf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            //let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
            let array = Uint8Array::new(&buf);
            let len = array.byte_length() as usize;
            super::imports::console_log(&format!("USDPL: Received websocket message with length {}", len));
            match socket::Packet::load(array.to_vec().as_slice()).0 {
                Some(socket::Packet::CallResponse(resp)) => {
                    let mut vec: Vec<JsValue> = Vec::with_capacity(resp.response.len());
                    for item in resp.response {
                        vec.push(super::convert::primitive_to_js(item));
                    }
                    let array: js_sys::Array = vec.iter().collect();
                    if let Err(e) = callback.call1(&JsValue::NULL, &array) {
                        imports::console_warn(&format!("USDPL warning: Callback error -- {:?}", e));
                    }
                },
                _ => {
                    imports::console_warn(&format!("USDPL warning: Got non-call-response message from {}", e.origin()));
                }
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

fn onopen_factory(buffer: ArrayBuffer, socket: WebSocket) -> impl FnMut(JsValue) {
    move |_| {
        imports::console_log("USDPL: connection opened");
        match socket.send_with_array_buffer(&buffer) {
            Ok(_) => {},
            Err(e) => {
                imports::console_error(&format!("USDPL error: socket send_with_array_buffer(...) failed -- {:?}", e));
            }
        }
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
}*/
