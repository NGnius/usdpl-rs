use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::io::{Read, Write};

use usdpl_core::serdes::{Dumpable, Loadable, Primitive};
use usdpl_core::{RemoteCallResponse, socket};

/// Instance for interacting with the front-end
pub struct Instance<'a> {
    calls: HashMap<String, &'a mut dyn FnMut(Vec<Primitive>) -> Vec<Primitive>>,
    port: u16,
}

impl<'a> Instance<'a> {
    #[inline]
    pub fn new(port_usdpl: u16) -> Self {
        Instance {
            calls: HashMap::new(),
            port: port_usdpl,
        }
    }

    /// Register a function which can be invoked by the front-end
    pub fn register<F: (FnMut(Vec<Primitive>) -> Vec<Primitive>) + Send + Sync>(&mut self, name: String, f: &'a mut F) -> &mut Self {
        self.calls.insert(name, f);
        self
    }

    fn handle_packet<const ERROR: bool>(&mut self, packet: socket::Packet, buffer: &mut [u8], incoming: &mut TcpStream) -> std::io::Result<()> {
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
                        return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Cannot dump return value of function `{}`", &obj.function)));
                    }
                    if ERROR {
                        incoming.write(&buffer[..len])?;
                    } else {
                        incoming.write(&buffer[..len]).unwrap_or_default();
                    }
                } else {
                    if ERROR {
                        return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Invalid remote call `{}` received from {}", obj.function, incoming.peer_addr()?)));
                    } else {
                        eprintln!("Invalid remote call `{}` received from {}", obj.function, incoming.peer_addr()?);
                    }

                }
            },
            socket::Packet::Many(many) => {
                for packet in many {
                    if let socket::Packet::Many(_) = packet {
                        // drop nested socket packets (prevents DoS and bad practices)
                        if ERROR {
                            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Invalid nested Many packet received from {}", incoming.peer_addr()?)));
                        } else {
                            eprintln!("Invalid nested Many packet received from {}", incoming.peer_addr()?);
                        }
                        continue;
                    }
                    self.handle_packet::<ERROR>(packet, buffer, incoming)?;
                }
            },
            _ => {
                let (ok, len) = socket::Packet::Unsupported.dump(buffer);
                if !ok && ERROR {
                    return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Cannot dump unsupported packet")));
                }
                if ERROR {
                    incoming.write(&buffer[..len])?;
                } else {
                    incoming.write(&buffer[..len]).unwrap_or_default();
                }
            }
        }
        Ok(())
    }

    pub fn serve<const ERROR: bool>(&mut self) -> std::io::Result<()> {
        let result = self.serve_internal::<ERROR>();
        //println!("Stopping server due to serve_internal returning a result");
        result
    }

    /// Receive and execute callbacks forever
    pub fn serve_internal<const ERROR: bool>(&mut self) -> std::io::Result<()> {
        let listener = TcpListener::bind(socket::socket_addr(self.port))?;
        for incoming in listener.incoming() {
            let mut incoming = incoming?;
            let mut buffer = [0u8; socket::PACKET_BUFFER_SIZE];
            let len = incoming.read(&mut buffer)?;
            let (obj_maybe, _) = socket::Packet::load(&buffer[..len]);
            if let Some(packet) = obj_maybe {
                self.handle_packet::<ERROR>(packet, &mut buffer, &mut incoming)?;
            } else {
                if ERROR {
                    return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("Invalid packet received from {}", incoming.peer_addr()?)));
                } else {
                    eprintln!("Invalid packet received from {}", incoming.peer_addr()?);
                }
            }
            incoming.shutdown(std::net::Shutdown::Both)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpStream;
    use super::*;

    const PORT: u16 = 31337;

    #[test]
    fn serve_full_test() -> std::io::Result<()> {
        let _server = std::thread::spawn(|| {
            Instance::new(PORT, PORT + 80)
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
        Instance::new(PORT+1, PORT+1+80)
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
            Instance::new(PORT+2, PORT+2+80)
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
    }
}
