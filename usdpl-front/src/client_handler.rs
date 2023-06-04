use std::sync::atomic::{AtomicU64, Ordering};

use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use nrpc::{ClientHandler, ServiceError, _helpers::async_trait, _helpers::bytes};
use wasm_bindgen_futures::spawn_local;

static LAST_ID: AtomicU64 = AtomicU64::new(0);

/// Websocket client.
/// In most cases, this shouldn't be used directly, but generated code will use this.
pub struct WebSocketHandler {
    port: u16,
}

async fn send_recv_ws(url: String, input: bytes::Bytes) -> Result<Vec<u8>, String> {
    let mut ws = WebSocket::open(&url).map_err(|e| e.to_string())?;
    ws.send(Message::Bytes(input.into()))
        .await
        .map_err(|e| e.to_string())?;

    read_next_incoming(ws).await
}

async fn read_next_incoming(mut ws: WebSocket) -> Result<Vec<u8>, String> {
    if let Some(msg) = ws.next().await {
        match msg.map_err(|e| e.to_string())? {
            Message::Bytes(b) => Ok(b),
            Message::Text(_) => Err("Message::Text not allowed".into()),
        }
    } else {
        Err("No response received".into())
    }
}

#[derive(Debug)]
struct ErrorStr(String);

impl std::fmt::Display for ErrorStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error message: {}", self.0)
    }
}

impl std::error::Error for ErrorStr {}

impl WebSocketHandler {
    /// Instantiate the web socket client for connecting on the specified port
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

#[async_trait::async_trait]
impl ClientHandler for WebSocketHandler {
    async fn call(
        &mut self,
        package: &str,
        service: &str,
        method: &str,
        input: bytes::Bytes,
        output: &mut bytes::BytesMut,
    ) -> Result<(), ServiceError> {
        let id = LAST_ID.fetch_add(1, Ordering::SeqCst);
        let url = format!(
            "ws://usdpl-ws-{}.localhost:{}/{}.{}/{}",
            id, self.port, package, service, method,
        );
        let (tx, rx) = async_channel::bounded(1);
        spawn_local(async move {
            tx.send(send_recv_ws(url, input).await).await.unwrap_or(());
        });

        output.extend_from_slice(
            &rx.recv()
                .await
                .map_err(|e| ServiceError::Method(Box::new(e)))?
                .map_err(|e| ServiceError::Method(Box::new(ErrorStr(e))))?,
        );
        Ok(())
    }
}
