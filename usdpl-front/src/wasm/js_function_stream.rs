use core::pin::Pin;
use core::future::Future;

use futures::{Stream, task::{Poll, Context}};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsValue;
use js_sys::{Function, Promise};

use nrpc::ServiceError;
use super::FromWasmStreamableType;
use crate::convert::js_to_str;

/// futures::Stream wrapper for a JS async function that generates a new T-like value every call
pub struct JsFunctionStream<T: FromWasmStreamableType + Unpin + 'static> {
    function: Function,
    promise: Option<JsFuture>,
    _idc: std::marker::PhantomData<T>,
}

impl <T: FromWasmStreamableType + Unpin + 'static> JsFunctionStream<T> {
    /// Construct the function stream wrapper
    pub fn from_function(f: Function) -> Self {
        Self {
            function: f,
            promise: None,
            _idc: std::marker::PhantomData::default(),
        }
    }
}

impl <T: FromWasmStreamableType  + Unpin + 'static> Stream for JsFunctionStream<T> {
    type Item = Result<T, ServiceError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {
        // this is horrible, I'm sorry
        let js_poll = if let Some(mut promise) = self.promise.take() {
            let mut pin = Pin::new(&mut promise);
            JsFuture::poll(pin.as_mut(), cx)
        } else {
            let function_result = match self.function.call0(&JsValue::undefined()) {
                Ok(x) => x,
                Err(e) => return Poll::Ready(Some(Err(ServiceError::Method(s_to_err(format!("JS function call error: {}", js_to_str(e)))))))
            };

            let js_promise = Promise::from(function_result);
            let mut js_future = JsFuture::from(js_promise);
            let mut pin = Pin::new(&mut js_future);
            let poll = JsFuture::poll(pin.as_mut(), cx);
            self.promise = Some(js_future);
            poll
        };
        js_poll.map(|t| match t {
            Ok(t) => {
                if t.is_null() || t.is_undefined() {
                    None
                } else {
                    Some(T::from_wasm_streamable(t).map_err(|e| ServiceError::Method(s_to_err(format!("JS type conversion error: {}", e)))))
                }
            },
            Err(e) => Some(Err(ServiceError::Method(s_to_err(format!("JS function promise error: {}", js_to_str(e))))))
        })
    }
}

fn s_to_err(s: String) -> Box<(dyn std::error::Error + Send + Sync + 'static)> {
    s.into()
}

fn _check_service_stream<T: FromWasmStreamableType  + Unpin + 'static>(js_stream: JsFunctionStream<T>) {
    let _: nrpc::ServiceClientStream<'static, T> = Box::new(js_stream);
}
