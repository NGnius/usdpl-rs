use bytes::BytesMut;
use ratchet_rs::deflate::DeflateExtProvider;
use ratchet_rs::{Error as RatchetError, Message, ProtocolRegistry, WebSocketConfig};
use tokio::net::{TcpListener, TcpStream};

use crate::rpc::ServiceRegistry;

struct MethodDescriptor<'a> {
    service: &'a str,
    method: &'a str,
}

/// Handler for communication to and from the front-end
pub struct WebsocketServer {
    services: ServiceRegistry<'static>,
    port: u16,
}

impl WebsocketServer {
    /// Initialise an instance of the back-end websocket server
    pub fn new(port_usdpl: u16) -> Self {
        Self {
            services: ServiceRegistry::new(),
            port: port_usdpl,
        }
    }

    /// Get the service registry that the server handles
    pub fn registry(&mut self) -> &'_ mut ServiceRegistry<'static> {
        &mut self.services
    }

    /// Register a nRPC service for this server to handle
    pub fn register<S: nrpc::ServerService + Send + 'static>(mut self, service: S) -> Self {
        self.services.register(service);
        self
    }

    /// Run the web server forever, asynchronously
    pub async fn run(&self) -> std::io::Result<()> {
        #[cfg(debug_assertions)]
        let addr = (std::net::Ipv4Addr::UNSPECIFIED, self.port);
        #[cfg(not(debug_assertions))]
        let addr = (std::net::Ipv4Addr::LOCALHOST, self.port);

        let tcp = TcpListener::bind(addr).await?;

        while let Ok((stream, _addr_do_not_use)) = tcp.accept().await {
            tokio::spawn(Self::connection_handler(self.services.clone(), stream));
        }

        Ok(())
    }

    #[cfg(feature = "blocking")]
    /// Run the server forever, blocking the current thread
    pub fn run_blocking(self) -> std::io::Result<()> {
        let runner = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        runner.block_on(self.run())
    }

    async fn connection_handler(
        services: ServiceRegistry<'static>,
        stream: TcpStream,
    ) -> Result<(), RatchetError> {
        let upgraded = ratchet_rs::accept_with(
            stream,
            WebSocketConfig::default(),
            DeflateExtProvider::default(),
            ProtocolRegistry::new(["usdpl-nrpc"])?,
        )
        .await?
        .upgrade()
        .await?;

        let request_path = upgraded.request.uri().path();

        let mut websocket = upgraded.websocket;

        let descriptor = Self::parse_uri_path(request_path)
            .map_err(|e| RatchetError::with_cause(ratchet_rs::ErrorKind::Protocol, e))?;

        let mut buf = BytesMut::new();
        loop {
            match websocket.read(&mut buf).await? {
                Message::Text => {
                    return Err(RatchetError::with_cause(
                        ratchet_rs::ErrorKind::Protocol,
                        "Websocket text messages are not accepted",
                    ))
                }
                Message::Binary => {
                    let response = services
                        .call_descriptor(
                            descriptor.service,
                            descriptor.method,
                            buf.clone().freeze(),
                        )
                        .await
                        .map_err(|e| {
                            RatchetError::with_cause(ratchet_rs::ErrorKind::Protocol, e.to_string())
                        })?;
                    websocket.write_binary(response).await?;
                }
                Message::Ping(x) => websocket.write_pong(x).await?,
                Message::Pong(_) => {}
                Message::Close(_) => break,
            }
        }
        Ok(())
    }

    fn parse_uri_path<'a>(path: &'a str) -> Result<MethodDescriptor<'a>, &'static str> {
        let mut iter = path.split('/');
        if let Some(service) = iter.next() {
            if let Some(method) = iter.next() {
                if iter.next().is_none() {
                    return Ok(MethodDescriptor { service, method });
                } else {
                    Err("URL path has too many separators")
                }
            } else {
                Err("URL path has no method")
            }
        } else {
            Err("URL path has no service")
        }
    }
}
