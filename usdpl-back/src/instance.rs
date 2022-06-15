//use std::net::{TcpListener, TcpStream, SocketAddr};
use std::collections::HashMap;
//use std::sync::Arc;
//use std::io::{Read, Write};

use lazy_static::lazy_static;

use warp::Filter;

use usdpl_core::serdes::{Dumpable, Loadable, Primitive};
use usdpl_core::{RemoteCallResponse, socket};

type Callable = Box<(dyn (Fn(Vec<Primitive>) -> Vec<Primitive>) + Send + Sync)>;

lazy_static! {
    static ref CALLS: std::sync::Mutex<HashMap<String, Callable>> = std::sync::Mutex::new(HashMap::new());
}

/// Back-end instance for interacting with the front-end
pub struct Instance {
    //calls: HashMap<String, Callable>,
    port: u16,
}

impl Instance {
    /// Initialise an instance of the back-end
    #[inline]
    pub fn new(port_usdpl: u16) -> Self {
        Instance {
            //calls: HashMap::new(),
            port: port_usdpl,
        }
    }

    /// Register a function which can be invoked by the front-end
    pub fn register<S: std::convert::Into<String>, F: (Fn(Vec<Primitive>) -> Vec<Primitive>) + Send + Sync + 'static>(&mut self, name: S, f: F) -> &mut Self {
        CALLS.lock().unwrap().insert(name.into(), Box::new(f));
        //self.calls.insert(name.into(), Box::new(f));
        self
    }

    /*fn handle_packet<const ERROR: bool>(&mut self, packet: socket::Packet, buffer: &mut [u8], peer_addr: &SocketAddr) -> Result<String, super::ServerError> {
        match packet {
            socket::Packet::Call(obj) => {
                if let Some(target_func) = self.calls.get_mut(&obj.function) {
                    // TODO: multithread this
                    let result = target_func(obj.parameters);
                    let response = socket::Packet::CallResponse(RemoteCallResponse {
                        id: obj.id,
                        response: result,
                    });
                    let (ok, len) = response.dump(buffer);
                    if !ok && ERROR {
                        return Err(super::ServerError::Io(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Cannot dump return value of function `{}`", &obj.function))));
                    }
                    if ERROR {
                        let mut vec = Vec::with_capacity(len);
                        vec.extend_from_slice(&buffer[..len]);
                        incoming.write_message(Message::Binary(vec)).map_err(super::ServerError::Tungstenite)?;
                    } else {
                        let mut vec = Vec::with_capacity(len);
                        vec.extend_from_slice(&buffer[..len]);
                        incoming.write_message(Message::Binary(vec)).unwrap_or_default();
                    }
                } else {
                    if ERROR {
                        return Err(super::ServerError::Io(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Invalid remote call `{}` received from {}", obj.function, peer_addr))));
                    } else {
                        eprintln!("Invalid remote call `{}` received from {}", obj.function, peer_addr);
                    }

                }
            },
            socket::Packet::Many(many) => {
                for packet in many {
                    if let socket::Packet::Many(_) = packet {
                        // drop nested socket packets (prevents DoS and bad practices)
                        if ERROR {
                            return Err(super::ServerError::Io(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Invalid nested Many packet received from {}", peer_addr))));
                        } else {
                            eprintln!("Invalid nested Many packet received from {}", peer_addr);
                        }
                        continue;
                    }
                    self.handle_packet::<ERROR>(packet, buffer, incoming, peer_addr)?;
                }
            },
            _ => {
                let (ok, len) = socket::Packet::Unsupported.dump(buffer);
                if !ok && ERROR {
                    return Err(super::ServerError::Io(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Cannot dump unsupported packet"))));
                }
                if ERROR {
                    let mut vec = Vec::with_capacity(len);
                    vec.extend_from_slice(&buffer[..len]);
                    incoming.write_message(Message::Binary(vec)).map_err(super::ServerError::Tungstenite)?;
                } else {
                    let mut vec = Vec::with_capacity(len);
                    vec.extend_from_slice(&buffer[..len]);
                    incoming.write_message(Message::Binary(vec)).unwrap_or_default();
                }
            }
        }
        Ok(())
    }*/

    pub fn serve(self) -> super::ServerResult {
        let result = self.serve_internal();
        //println!("Stopping server due to serve_internal returning a result");
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(result)
    }

    fn handle_call(packet: socket::Packet) -> socket::Packet {
        println!("Got packet");
        match packet {
            socket::Packet::Call(call) => {
                let handlers = CALLS.lock().expect("Failed to acquite CALLS lock");
                if let Some(target) = handlers.get(&call.function) {
                    let result = target(call.parameters);
                    socket::Packet::CallResponse(RemoteCallResponse {
                        id: call.id,
                        response: result,
                    })
                } else {
                    socket::Packet::Invalid
                }
            },
            socket::Packet::Many(packets) => {
                let mut result = Vec::with_capacity(packets.len());
                for packet in packets {
                    result.push(Self::handle_call(packet));
                }
                socket::Packet::Many(result)
            },
            _ => socket::Packet::Invalid,
        }
    }

    /// Receive and execute callbacks forever
    pub async fn serve_internal(self) -> super::ServerResult {
        //let handlers = self.calls;
        //self.calls = HashMap::new();
        let calls = warp::post()
            .and(warp::path("usdpl/call"))
            .and(warp::body::content_length_limit((socket::PACKET_BUFFER_SIZE * 2) as _))
            .and(warp::body::bytes())
            .map(|data: bytes::Bytes| {
                let (obj_maybe, _) = socket::Packet::load_base64(&data);
                let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
                if let Some(packet) = obj_maybe {
                    let response = Self::handle_call(packet);
                    let (ok, len) = response.dump_base64(&mut buffer);
                    if !ok {
                        eprintln!("Failed to dump response packet");
                        warp::reply::with_status(warp::http::Response::builder()
                            .body("Failed to dump response packet".to_string()), warp::http::StatusCode::from_u16(400).unwrap())
                    } else {
                        let string: String = String::from_utf8_lossy(&buffer[..len]).into();
                        warp::reply::with_status(warp::http::Response::builder()
                            .body(string), warp::http::StatusCode::from_u16(200).unwrap())
                    }
                } else {
                    eprintln!("Failed to load packet");
                    warp::reply::with_status(warp::http::Response::builder()
                        .body("Failed to load packet".to_string()), warp::http::StatusCode::from_u16(400).unwrap())
                }
            })
            .map(|reply| {
                warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*")
            });

        warp::serve(calls).run(([127, 0, 0, 1], self.port)).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //use std::net::TcpStream;
    //use super::*;

    //const PORT: u16 = 31337;

    /*#[test]
    fn serve_full_test() -> std::io::Result<()> {
        let _server = std::thread::spawn(|| {
            Instance::new(PORT)
                .register("echo".to_string(), &mut |params| params)
                .register("hello".to_string(), &mut |params| {
                    if let Some(Primitive::String(name)) = params.get(0) {
                        vec![Primitive::String(format!("Hello {}", name))]
                    } else {
                        vec![]
                    }
                })
                .serve::<true>()
        });
        std::thread::sleep(std::time::Duration::from_millis(10));
        let mut front = TcpStream::connect(socket::socket_addr(PORT)).unwrap();
        let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
        let call = socket::Packet::Call(usdpl_core::RemoteCall {
            id: 42,
            function: "hello".to_string(),
            parameters: vec![Primitive::String("USDPL".to_string())]
        });
        let (ok, len) = call.dump(&mut buffer);
        assert!(ok, "Packet dump failed");
        assert_eq!(len, 32, "Packet dumped wrong amount of data");
        front.write(&buffer[..len])?;
        let len = front.read(&mut buffer)?;
        let (response, len) = socket::Packet::load(&buffer[..len]);
        assert!(response.is_some(), "Response load failed");
        assert_eq!(len, 29, "Response loaded wrong amount of data");
        let response = response.unwrap();
        if let socket::Packet::CallResponse(resp) = response {
            assert_eq!(resp.id, 42);
            if let Some(Primitive::String(s)) = resp.response.get(0) {
                assert_eq!(s, "Hello USDPL");
            } else {
                panic!("Wrong response data");
            }
        } else {
            panic!("Wrong response packet type");
        }

        Ok(())
    }

    #[test]
    #[should_panic]
    fn serve_err_test() {
        let _client = std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let mut front = TcpStream::connect(socket::socket_addr(PORT+1)).unwrap();
            let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
            let (_, len) = socket::Packet::Bad.dump(&mut buffer);
            front.write(&buffer[..len]).unwrap();
            let _ = front.read(&mut buffer).unwrap();
        });
        Instance::new(PORT+1)
            .register("echo".to_string(), &mut |params| params)
            .register("hello".to_string(), &mut |params| {
                if let Some(Primitive::String(name)) = params.get(0) {
                    vec![Primitive::String(format!("Hello {}", name))]
                } else {
                    vec![]
                }
            })
            .serve::<true>()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn serve_unsupported_test() {
        let _server = std::thread::spawn(|| {
            Instance::new(PORT+2)
                .register("echo".to_string(), &mut |params| params)
                .register("hello".to_string(), &mut |params| {
                    if let Some(Primitive::String(name)) = params.get(0) {
                        vec![Primitive::String(format!("Hello {}", name))]
                    } else {
                        vec![]
                    }
                })
                .serve::<true>()
        });
        std::thread::sleep(std::time::Duration::from_millis(10));
        let mut front = TcpStream::connect(socket::socket_addr(PORT+2)).unwrap();
        let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
        let (ok, len) = socket::Packet::Unsupported.dump(&mut buffer);
        assert!(ok, "Packet dump failed");
        assert_eq!(len, 32, "Packet dumped wrong amount of data");
        front.write(&buffer[..len]).unwrap();
        let len = front.read(&mut buffer).unwrap();
        let (response, len) = socket::Packet::load(&buffer[..len]);
        assert!(response.is_some(), "Response load failed");
        assert_eq!(len, 29, "Response loaded wrong amount of data");
        let response = response.unwrap();
        if let socket::Packet::Unsupported = response {
        } else {
            panic!("Wrong response packet type");
        }
    }*/
}
