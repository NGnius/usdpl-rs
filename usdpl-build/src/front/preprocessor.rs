use nrpc_build::IPreprocessor;
//use prost_build::{Service, ServiceGenerator};
use prost_types::FileDescriptorSet;

use super::SharedState;

pub struct WasmProtoPreprocessor {
    shared: SharedState,
}

impl WasmProtoPreprocessor {
    pub fn with_state(state: &SharedState) -> Self {
        Self {
            shared: state.clone(),
        }
    }
}

impl IPreprocessor for WasmProtoPreprocessor {
    fn process(&mut self, fds: &mut FileDescriptorSet) -> proc_macro2::TokenStream {
        self.shared.lock().expect("Cannot lock shared state").fds = Some(fds.clone());
        quote::quote! {}
    }
}
