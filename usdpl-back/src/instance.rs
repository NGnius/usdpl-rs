use std::collections::HashMap;

use warp::Filter;

use usdpl_core::serdes::{Dumpable, Loadable};
use usdpl_core::{socket, RemoteCallResponse};

use super::{Callable, MutCallable, AsyncCallable, WrappedCallable};

//type WrappedCallable = Arc<Mutex<Box<dyn Callable>>>; // thread-safe, cloneable Callable

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

    /// Register a thread-safe function which can be invoked by the front-end
    pub fn register<S: std::convert::Into<String>, F: Callable + 'static>(
        mut self,
        name: S,
        f: F,
    ) -> Self {
        self.calls
            .insert(name.into(), WrappedCallable::new_ref(f));
        self
    }

    /// Register a thread-unsafe function which can be invoked by the front-end
    pub fn register_blocking<S: std::convert::Into<String>, F: MutCallable + 'static>(
        mut self,
        name: S,
        f: F,
    ) -> Self {
        self.calls
            .insert(name.into(), WrappedCallable::new_locking(f));
        self
    }

    /// Register a thread-unsafe function which can be invoked by the front-end
    pub fn register_async<S: std::convert::Into<String>, F: AsyncCallable + 'static>(
        mut self,
        name: S,
        f: F,
    ) -> Self {
        self.calls
            .insert(name.into(), WrappedCallable::new_async(f));
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

    #[async_recursion::async_recursion]
    async fn handle_call(
        packet: socket::Packet,
        handlers: &HashMap<String, WrappedCallable>,
    ) -> socket::Packet {
        match packet {
            socket::Packet::Call(call) => {
                log::info!("Got USDPL call {} (`{}`, params: {})", call.id, call.function, call.parameters.len());
                //let handlers = CALLS.lock().expect("Failed to acquire CALLS lock");
                if let Some(target) = handlers.get(&call.function) {
                    let result = target.call(call.parameters).await;
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
                    result.push(Self::handle_call(packet, handlers).await);
                }
                socket::Packet::Many(result)
            }
            _ => socket::Packet::Invalid,
        }
    }

    #[cfg(not(feature = "encrypt"))]
    async fn process_body((data, handlers): (bytes::Bytes, HashMap<String, WrappedCallable>)) -> impl warp::Reply {
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
        let response = Self::handle_call(packet, &handlers).await;
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
    }

    #[cfg(feature = "encrypt")]
    async fn process_body((data, handlers, key): (bytes::Bytes, HashMap<String, WrappedCallable>, Vec<u8>)) -> impl warp::Reply {
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
        let response = Self::handle_call(packet, &handlers).await;
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
    }

    /// Receive and execute callbacks forever
    async fn serve_internal(&self) -> Result<(), ()> {
        let handlers = self.calls.clone();
        #[cfg(not(feature = "encrypt"))]
        let input_mapper = move |data: bytes::Bytes| { (data, handlers.clone()) };
        #[cfg(feature = "encrypt")]
        let key = self.encryption_key.clone();
        #[cfg(feature = "encrypt")]
        let input_mapper = move |data: bytes::Bytes| { (data, handlers.clone(), key.clone()) };
        //self.calls = HashMap::new();
        let calls = warp::post()
            .and(warp::path!("usdpl" / "call"))
            .and(warp::body::content_length_limit(
                (socket::PACKET_BUFFER_SIZE * 2) as _,
            ))
            .and(warp::body::bytes())
            .map(input_mapper)
            .then(Self::process_body)
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
