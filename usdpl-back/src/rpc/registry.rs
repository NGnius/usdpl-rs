use std::collections::HashMap;
use std::sync::Arc;
use async_lock::Mutex;

use nrpc::{ServerService, ServiceError};

#[derive(Default, Clone)]
pub struct ServiceRegistry<'a> {
    entries: HashMap<String, Arc<Mutex<Box<dyn ServerService + Send + 'a>>>>,
}

impl<'a> ServiceRegistry<'a> {
    /*pub async fn call(&self, package: &str, service: &str, method: &str, data: bytes::Bytes) -> Result<bytes::Bytes, ServiceError> {
        let key = Self::descriptor(package, service);
        self.call_descriptor(&key, method, data).await
    }

    fn descriptor(package: &str, service: &str) -> String {
        format!("{}.{}", package, service)
    }*/

    pub async fn call_descriptor(&self, descriptor: &str, method: &str, data: bytes::Bytes) -> Result<bytes::Bytes, ServiceError> {
        if let Some(service) = self.entries.get(descriptor) {
            let mut output = bytes::BytesMut::new();
            let mut service_lock = service.lock_arc().await;
            service_lock.call(method, data, &mut output).await?;
            Ok(output.into())
        } else {
            Err(ServiceError::ServiceNotFound)
        }
    }

    pub fn register<S: ServerService + Send + 'a>(&mut self, service: S) -> &mut Self {
        let key = service.descriptor().to_owned();
        self.entries.insert(key, Arc::new(Mutex::new(Box::new(service))));
        self
    }

    pub fn new() -> Self {
        Self::default()
    }
}
