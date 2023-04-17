use std::sync::{Arc, Mutex};

use prost_types::FileDescriptorSet;

#[derive(Clone)]
pub struct SharedState(Arc<Mutex<SharedProtoData>>);

impl SharedState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(SharedProtoData {
            fds: None,
        })))
    }
}

impl std::ops::Deref for SharedState {
    type Target = Arc<Mutex<SharedProtoData>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct SharedProtoData {
    pub fds: Option<FileDescriptorSet>,
}
