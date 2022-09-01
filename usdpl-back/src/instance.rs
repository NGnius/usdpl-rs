use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use warp::Filter;

use usdpl_core::serdes::{Dumpable, Loadable};
use usdpl_core::{socket, RemoteCallResponse};

use super::Callable;

type WrappedCallable = Arc<Mutex<Box<dyn Callable>>>; // thread-safe, cloneable Callable

#[cfg(feature = "encrypt")]
const NONCE: [u8; socket::NONCE_SIZE] = [0u8; socket::NONCE_SIZE];

/// Back-end instance for interacting with the front-end
pub struct Instance {
    calls: HashMap<String, WrappedCallable>,
    port: u16,
    #[cfg(feature = "encrypt")]
    encryption_key: Vec<u8>,
}

impl Instance {
    /// Initialise an instance of the back-end
    #[inline]
    pub fn new(port_usdpl: u16) -> Self {
        Instance {
            calls: HashMap::new(),
            port: port_usdpl,
            #[cfg(feature = "encrypt")]
            encryption_key: hex::decode(obfstr::obfstr!(env!("USDPL_ENCRYPTION_KEY"))).unwrap(),
        }
    }

    /// Register a function which can be invoked by the front-end, builder style
    pub fn register<S: std::convert::Into<String>, F: Callable + 'static>(
        mut self,
        name: S,
        f: F,
    ) -> Self {
        self.calls
            .insert(name.into(), Arc::new(Mutex::new(Box::new(f))));
        self
    }

    /// Register a function which can be invoked by the front-end, object style
    pub fn register_mut<S: std::convert::Into<String>, F: Callable + 'static>(
        &mut self,
        name: S,
        f: F,
    ) -> &mut Self {
        self.calls
            .insert(name.into(), Arc::new(Mutex::new(Box::new(f))));
        self
    }

    /// Run the web server instance forever, blocking this thread
    #[cfg(feature = "blocking")]
    pub fn run_blocking(&self) -> Result<(), ()> {
        let result = self.serve_internal();
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(result)
    }

    /// Run the web server forever, asynchronously
    pub async fn run(&self) -> Result<(), ()> {
        self.serve_internal().await
    }

    fn handle_call(
        packet: socket::Packet,
        handlers: &HashMap<String, WrappedCallable>,
    ) -> socket::Packet {
        match packet {
            socket::Packet::Call(call) => {
                //let handlers = CALLS.lock().expect("Failed to acquire CALLS lock");
                if let Some(target) = handlers.get(&call.function) {
                    let result = target
                        .lock()
                        .expect("Failed to acquire CALLS.function lock")
                        .call(call.parameters);
                    socket::Packet::CallResponse(RemoteCallResponse {
                        id: call.id,
                        response: result,
                    })
                } else {
                    socket::Packet::Invalid
                }
            }
            socket::Packet::Many(packets) => {
                let mut result = Vec::with_capacity(packets.len());
                for packet in packets {
                    result.push(Self::handle_call(packet, handlers));
                }
                socket::Packet::Many(result)
            }
            _ => socket::Packet::Invalid,
        }
    }

    /// Receive and execute callbacks forever
    async fn serve_internal(&self) -> Result<(), ()> {
        let handlers = self.calls.clone();
        //self.calls = HashMap::new();
        #[cfg(not(feature = "encrypt"))]
        let calls = warp::post()
            .and(warp::path!("usdpl" / "call"))
            .and(warp::body::content_length_limit(
                (socket::PACKET_BUFFER_SIZE * 2) as _,
            ))
            .and(warp::body::bytes())
            .map(move |data: bytes::Bytes| {
                let (packet, _) = match socket::Packet::load_base64(&data) {
                    Ok(x) => x,
                    Err(e) => {
                        return warp::reply::with_status(
                            warp::http::Response::builder()
                                .body(format!("Failed to load packet: {}", e)),
                            warp::http::StatusCode::from_u16(400).unwrap(),
                        )
                    }
                };
                //let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
                let mut buffer = String::with_capacity(socket::PACKET_BUFFER_SIZE);
                let response = Self::handle_call(packet, &handlers);
                let _len = match response.dump_base64(&mut buffer) {
                    Ok(x) => x,
                    Err(e) => {
                        return warp::reply::with_status(
                            warp::http::Response::builder()
                                .body(format!("Failed to dump response packet: {}", e)),
                            warp::http::StatusCode::from_u16(500).unwrap(),
                        )
                    }
                };
                warp::reply::with_status(
                    warp::http::Response::builder().body(buffer),
                    warp::http::StatusCode::from_u16(200).unwrap(),
                )
            })
            .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*"));
        #[cfg(feature = "encrypt")]
        let key = self.encryption_key.clone();
        #[cfg(feature = "encrypt")]
        let calls = warp::post()
            .and(warp::path!("usdpl" / "call"))
            .and(warp::body::content_length_limit(
                (socket::PACKET_BUFFER_SIZE * 2) as _,
            ))
            .and(warp::body::bytes())
            .map(move |data: bytes::Bytes| {
                let (packet, _) = match socket::Packet::load_encrypted(&data, &key, &NONCE) {
                    Ok(x) => x,
                    Err(_) => {
                        return warp::reply::with_status(
                            warp::http::Response::builder()
                                .body("Failed to load packet".to_string()),
                            warp::http::StatusCode::from_u16(400).unwrap(),
                        )
                    }
                };
                let mut buffer = Vec::with_capacity(socket::PACKET_BUFFER_SIZE);
                //buffer.extend(&[0u8; socket::PACKET_BUFFER_SIZE]);
                let response = Self::handle_call(packet, &handlers);
                let len = match response.dump_encrypted(&mut buffer, &key, &NONCE) {
                    Ok(x) => x,
                    Err(_) => {
                        return warp::reply::with_status(
                            warp::http::Response::builder()
                                .body("Failed to dump response packet".to_string()),
                            warp::http::StatusCode::from_u16(500).unwrap(),
                        )
                    }
                };
                buffer.truncate(len);
                let string: String = String::from_utf8(buffer).unwrap().into();
                warp::reply::with_status(
                    warp::http::Response::builder().body(string),
                    warp::http::StatusCode::from_u16(200).unwrap(),
                )
            })
            .map(|reply| warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*"));
        #[cfg(debug_assertions)]
        warp::serve(calls).run(([0, 0, 0, 0], self.port)).await;
        #[cfg(not(debug_assertions))]
        warp::serve(calls).run(([127, 0, 0, 1], self.port)).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
}
