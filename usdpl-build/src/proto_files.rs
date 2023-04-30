use std::path::{Path, PathBuf};

struct IncludedFileStr<'a> {
    filename: &'a str,
    contents: &'a str,
}

const ADDITIONAL_PROTOBUFS_ENV_VAR: &'static str = "USDPL_PROTOS_PATH";

const DEBUG_PROTO: IncludedFileStr<'static> = IncludedFileStr {
    filename: "debug.proto",
    contents: include_str!("../protos/debug.proto"),
};

const TRANSLATIONS_PROTO: IncludedFileStr<'static> = IncludedFileStr {
    filename: "translations.proto",
    contents: include_str!("../protos/translations.proto"),
};

const ALL_PROTOS: [IncludedFileStr<'static>; 2] = [
    DEBUG_PROTO,
    TRANSLATIONS_PROTO,
];

pub fn proto_builtins_out_path() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").expect("Not in a build.rs context (missing $OUT_DIR)")).join("protos")
}

pub fn proto_out_paths() -> impl Iterator<Item = String> {
    std::iter::once(proto_builtins_out_path())
        .map(|x| x.to_str().unwrap().to_owned())
        .chain(custom_protos_dirs().into_iter())
}

fn custom_protos_dirs() -> Vec<String> {
    let dirs = std::env::var(ADDITIONAL_PROTOBUFS_ENV_VAR).unwrap_or_else(|_| "".to_owned());
    dirs.split(':')
        .filter(|x| std::fs::read_dir(x).is_ok())
        .map(|x| x.to_owned())
        .collect()
}

fn custom_protos_filenames() -> Vec<String> {
    let dirs = std::env::var(ADDITIONAL_PROTOBUFS_ENV_VAR).unwrap_or_else(|_| "".to_owned());
    dirs.split(':')
        .map(std::fs::read_dir)
        .filter(|x| x.is_ok())
        .flat_map(|x| x.unwrap())
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap().path())
        .filter(|x| if let Some(ext) = x.extension() { ext.to_ascii_lowercase() == "proto" && x.is_file() } else { false })
        .filter_map(|x| x.to_str().map(|x| x.to_owned()))
        .collect()
}

pub fn all_proto_filenames(p: impl AsRef<Path> + 'static) -> impl Iterator<Item = String> {
    //let p = p.as_ref();
    ALL_PROTOS.iter().map(move |x| p.as_ref().join(x.filename).to_str().unwrap().to_owned()).chain(custom_protos_filenames())
}

pub fn dump_protos(p: impl AsRef<Path>) -> std::io::Result<()> {
    let p = p.as_ref();
    for f in ALL_PROTOS {
        let fullpath = p.join(f.filename);
        std::fs::write(fullpath, f.contents)?;
    }
    Ok(())
}

pub fn dump_protos_out() -> std::io::Result<()> {
    let path = proto_builtins_out_path();
    std::fs::create_dir_all(&path)?;
    dump_protos(&path)
}
