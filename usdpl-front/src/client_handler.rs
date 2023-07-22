use std::sync::atomic::{AtomicU64, Ordering};

use futures::{SinkExt, StreamExt, future::{select, Either}};
use gloo_net::websocket::{futures::WebSocket, Message, State};
use nrpc::{ClientHandler, ServiceError, ServiceClientStream, _helpers::async_trait, _helpers::bytes};
use wasm_bindgen_futures::spawn_local;

static LAST_ID: AtomicU64 = AtomicU64::new(0);

/// Websocket client.
/// In most cases, this shouldn't be used directly, but generated code will use this.
pub struct WebSocketHandler {
    port: u16,
}

async fn send_recv_ws<'a>(tx: async_channel::Sender<Result<bytes::Bytes, String>>, url: String, mut input: ServiceClientStream<'a, bytes::Bytes>) {
    let ws = match WebSocket::open(&url).map_err(|e| e.to_string()) {
        Ok(x) => x,
        Err(e) => {
            tx.send(Err(e.to_string())).await.unwrap_or(());
            return;
        }
    };

    let (mut input_done, mut output_done) = (false, false);
    let mut last_ws_state = ws.state();
    let (mut ws_sink, mut ws_stream) = ws.split();
    let (mut left, mut right) = (input.next(), ws_stream.next());
    while let State::Open = last_ws_state {
        if !input_done && !output_done {
            match select(left, right).await {
                Either::Left((next, outstanding)) => {
                    if let Some(next) = next {
                        match next {
                            Ok(next) => {
                                if let Err(e) = ws_sink.send(Message::Bytes(next.into())).await {
                                    tx.send(Err(e.to_string())).await.unwrap_or(());
                                }
                            },
                            Err(e) => tx.send(Err(e.to_string())).await.unwrap_or(())
                        }
                    } else {
                        input_done = true;
                    }
                    right = outstanding;
                    left = input.next();
                },
                Either::Right((response, outstanding)) => {
                    if let Some(next) = response {
                        match next {
                            Ok(Message::Bytes(b)) => tx.send(Ok(b.into())).await.unwrap_or(()),
                            Ok(_) => tx.send(Err("Message::Text not allowed".into())).await.unwrap_or(()),
                            Err(e) => tx.send(Err(e.to_string())).await.unwrap_or(()),
                        }
                    } else {
                        output_done = true;
                    }
                    left = outstanding;
                    let ws = ws_stream.reunite(ws_sink).unwrap();
                    last_ws_state = ws.state();
                    (ws_sink, ws_stream) = ws.split();
                    right = ws_stream.next();
                }
            }
        } else if input_done {
            if let Some(next) = right.await {
                match next {
                    Ok(Message::Bytes(b)) => tx.send(Ok(b.into())).await.unwrap_or(()),
                    Ok(_) => tx.send(Err("Message::Text not allowed".into())).await.unwrap_or(()),
                    Err(e) => tx.send(Err(e.to_string())).await.unwrap_or(()),
                }
            } else {
                output_done = true;
            }
            //left = outstanding;
            let ws = ws_stream.reunite(ws_sink).unwrap();
            last_ws_state = ws.state();
            (ws_sink, ws_stream) = ws.split();
            right = ws_stream.next();
        } else {

        }
    }
    /*spawn_local(async move {
        while let State::Open = ws.state() {
            if let Some(next) = input.next().await {
                match next {
                    Ok(next) => {
                        if let Err(e) = ws.send(Message::Bytes(next.into())).await {
                            tx2.send(Err(e.to_string())).await.unwrap_or(());
                        }
                    },
                    Err(e) => tx2.send(Err(e.to_string())).await.unwrap_or(())
                }
            } else {
                break;
            }
        }
    });

    spawn_local(async move {
        while let State::Open = ws.state() {
            if let Some(next) = ws.next().await {
                match next {
                    Ok(Message::Bytes(b)) => tx.send(Ok(b.into())).await.unwrap_or(()),
                    Ok(_) => tx.send(Err("Message::Text not allowed".into())).await.unwrap_or(()),
                    Err(e) => tx.send(Err(e.to_string())).await.unwrap_or(()),
                }
            } else {
                break;
            }
        }
    });*/
}

#[derive(Debug)]
struct ErrorStr(String);

impl std::fmt::Display for ErrorStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error message: {}", self.0)
    }
}

impl std::error::Error for ErrorStr {}

const CHANNEL_BOUND: usize = 4;

impl WebSocketHandler {
    /// Instantiate the web socket client for connecting on the specified port
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

#[async_trait::async_trait(?Send)]
impl ClientHandler<'static> for WebSocketHandler {
    async fn call<'a: 'static>(
        &self,
        package: &str,
        service: &str,
        method: &str,
        input: ServiceClientStream<'a, bytes::Bytes>,
    ) -> Result<ServiceClientStream<'a, bytes::Bytes>, ServiceError> {
        let id = LAST_ID.fetch_add(1, Ordering::SeqCst);
        let url = format!(
            "ws://usdpl-ws-{}.localhost:{}/{}.{}/{}",
            id, self.port, package, service, method,
        );
        let (tx, rx) = async_channel::bounded(CHANNEL_BOUND);
        spawn_local(send_recv_ws(tx, url, input));

        Ok(Box::new(rx.map(|buf_result: Result<bytes::Bytes, String>| buf_result
            .map(|buf| bytes::Bytes::from(buf))
            .map_err(|e| ServiceError::Method(Box::new(ErrorStr(e)))))))
    }
}
