pub fn build(
    custom_protos: impl Iterator<Item = String>,
    custom_dirs: impl Iterator<Item = String>,
) {
    crate::dump_protos_out().unwrap();
    nrpc_build::compile_servers(
        crate::all_proto_filenames(crate::proto_builtins_out_path(), custom_protos),
        crate::proto_out_paths(custom_dirs),
    )
}
