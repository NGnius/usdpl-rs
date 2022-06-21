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

pub async fn send_js(packet: socket::Packet, port: u16) -> Result<Vec<Primitive>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    let url = format!("http://{}:{}/usdpl/call", socket::HOST_STR, port);

    let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
    let len = packet
        .dump_base64(&mut buffer)
        .map_err(super::convert::str_to_js)?;
    let string: String = String::from_utf8_lossy(&buffer[..len]).into();
    opts.body(Some(&string.into()));

    let request = Request::new_with_str_and_init(&url, &opts)?;

    //request.headers().set("Accept", "text/base64")?;
    //.set("Authorization", "wasm TODO_KEY")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;
    let text = JsFuture::from(resp.text()?).await?;
    let string: JsString = text.dyn_into()?;

    match socket::Packet::load_base64(string.as_string().unwrap().as_bytes())
        .map_err(super::convert::str_to_js)?
        .0
    {
        socket::Packet::CallResponse(resp) => Ok(resp.response),
        _ => {
            //imports::console_warn(&format!("USDPL warning: Got non-call-response message from {}", resp.url()));
            Err(format!(
                "Expected call response message from {}, got something else",
                resp.url()
            )
            .into())
        }
    }
}
