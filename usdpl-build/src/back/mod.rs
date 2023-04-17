pub fn build() {
    crate::dump_protos_out().unwrap();
    nrpc_build::compile_servers(
        crate::all_proto_filenames().map(|n| crate::proto_out_path().clone().join(n)),
        [crate::proto_out_path()]
    )
}
