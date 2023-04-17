mod preprocessor;
pub use preprocessor::WasmProtoPreprocessor;

mod service_generator;
pub use service_generator::WasmServiceGenerator;

mod shared_state;
pub(crate) use shared_state::SharedState;

pub fn build() {
    let shared_state = SharedState::new();
    crate::dump_protos_out().unwrap();
    nrpc_build::Transpiler::new(
        crate::all_proto_filenames().map(|n| crate::proto_out_path().clone().join(n)),
        [crate::proto_out_path()]
    ).unwrap()
        .generate_client()
        .with_preprocessor(nrpc_build::AbstractImpl::outer(WasmProtoPreprocessor::with_state(&shared_state)))
        .with_service_generator(nrpc_build::AbstractImpl::outer(WasmServiceGenerator::with_state(&shared_state)))
        .transpile()
        .unwrap()
}
