use std::path::{Path, PathBuf};

struct IncludedFileStr<'a> {
    filename: &'a str,
    contents: &'a str,
}

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

pub fn proto_out_path() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").expect("Not in a build.rs context (missing $OUT_DIR)")).join("protos")
}

pub fn all_proto_filenames() -> impl Iterator<Item = &'static str> {
    ALL_PROTOS.iter().map(|x| x.filename)
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
    let path = proto_out_path();
    std::fs::create_dir_all(&path)?;
    dump_protos(&path)
}
