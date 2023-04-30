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
        crate::all_proto_filenames(crate::proto_builtins_out_path()),
        crate::proto_out_paths()
    ).unwrap()
        .generate_client()
        .with_preprocessor(nrpc_build::AbstractImpl::outer(WasmProtoPreprocessor::with_state(&shared_state)))
        .with_service_generator(nrpc_build::AbstractImpl::outer(WasmServiceGenerator::with_state(&shared_state)))
        .transpile()
        .unwrap()
}
