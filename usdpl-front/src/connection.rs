//use std::net::TcpStream;
//use std::io::{Read, Write};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

//use web_sys::{WebSocket, MessageEvent, ErrorEvent};
use js_sys::JsString;
use web_sys::{Request, RequestInit, RequestMode, Response};
//use wasm_rs_shared_channel::{Expects, spsc::{Receiver, Sender}};

use usdpl_core::serdes::{Dumpable, Loadable, Primitive};
use usdpl_core::socket;

#[cfg(feature = "encrypt")]
const NONCE: [u8; socket::NONCE_SIZE] = [0u8; socket::NONCE_SIZE];

pub async fn send_recv_packet(
    id: u64,
    packet: socket::Packet,
    port: u16,
    #[cfg(feature = "encrypt")] key: Vec<u8>,
) -> Result<socket::Packet, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let url = format!(
        "http://usdpl{}.{}:{}/usdpl/call",
        id,
        socket::HOST_STR,
        port
    );

    #[allow(unused_variables)]
    let (buffer, len) = dump_to_buffer(
        packet,
        #[cfg(feature = "encrypt")]
        key.as_slice(),
    )?;
    let string: String = String::from_utf8_lossy(buffer.as_slice()).into();
    #[cfg(feature = "debug")]
    crate::imports::console_log(&format!("Dumped base64 `{}` len:{}", string, len));
    opts.body(Some(&string.into()));

    let request = Request::new_with_str_and_init(&url, &opts)?;

    //request.headers().set("Accept", "text/base64")?;
    //.set("Authorization", "wasm TODO_KEY")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;
    let text = JsFuture::from(resp.text()?).await?;
    let string: JsString = text.dyn_into()?;

    let rust_str = string.as_string().unwrap();
    #[cfg(feature = "debug")]
    crate::imports::console_log(&format!(
        "Received base64 `{}` len:{}",
        rust_str,
        rust_str.len()
    ));

    #[cfg(not(feature = "encrypt"))]
    {
        Ok(socket::Packet::load_base64(rust_str.as_bytes())
            .map_err(super::convert::str_to_js)?
            .0)
    }

    #[cfg(feature = "encrypt")]
    {
        Ok(
            socket::Packet::load_encrypted(rust_str.as_bytes(), key.as_slice(), &NONCE)
                .map_err(super::convert::str_to_js)?
                .0,
        )
    }
}

pub async fn send_call(
    id: u64,
    packet: socket::Packet,
    port: u16,
    #[cfg(feature = "encrypt")] key: Vec<u8>,
) -> Result<Vec<Primitive>, JsValue> {
    let packet = send_recv_packet(
        id,
        packet,
        port,
        #[cfg(feature = "encrypt")]
        key,
    )
    .await?;

    match packet {
        socket::Packet::CallResponse(resp) => Ok(resp.response),
        _ => {
            //imports::console_warn(&format!("USDPL warning: Got non-call-response message from {}", resp.url()));
            Err("Expected call response message, got something else".into())
        }
    }
}

#[cfg(feature = "encrypt")]
fn dump_to_buffer(packet: socket::Packet, key: &[u8]) -> Result<(Vec<u8>, usize), JsValue> {
    let mut buffer = Vec::with_capacity(socket::PACKET_BUFFER_SIZE);
    //buffer.extend_from_slice(&[0u8; socket::PACKET_BUFFER_SIZE]);
    let len = packet
        .dump_encrypted(&mut buffer, key, &NONCE)
        .map_err(super::convert::str_to_js)?;
    Ok((buffer, len))
}

#[cfg(not(feature = "encrypt"))]
fn dump_to_buffer(packet: socket::Packet) -> Result<(Vec<u8>, usize), JsValue> {
    let mut buffer = String::with_capacity(socket::PACKET_BUFFER_SIZE);
    //buffer.extend_from_slice(&[0u8; socket::PACKET_BUFFER_SIZE]);
    let len = packet
        .dump_base64(&mut buffer)
        .map_err(super::convert::str_to_js)?;
    Ok((buffer.as_bytes().to_vec(), len))
}
