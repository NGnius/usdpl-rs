pub fn build() {
    crate::dump_protos_out().unwrap();
    nrpc_build::compile_servers(
        crate::all_proto_filenames(crate::proto_builtins_out_path()),
        crate::proto_out_paths()
    )
}
