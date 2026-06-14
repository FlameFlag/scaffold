use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn main() {
    let root = PathBuf::from("src").join("extensions");
    println!("cargo:rerun-if-changed={}", root.display());

    let mut files = Vec::new();
    collect_scheme_files(&root, &mut files);
    files.sort();

    let mut entries = String::new();
    for path in &files {
        let path = path.to_string_lossy().replace('\\', "/");
        writeln!(
            &mut entries,
            "    BundledSchemeSource {{ path: {path:?}, source: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/\", {path:?})) }},"
        )
        .expect("write bundled extension source entry");
    }

    let generated = format!(
        "pub(super) struct BundledSchemeSource {{\n    pub(super) path: &'static str,\n    pub(super) source: &'static str,\n}}\n\npub(super) const BUNDLED_EXTENSION_SOURCES: &[BundledSchemeSource] = &[\n{entries}];\n"
    );

    let out_path =
        PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR")).join("bundled_extensions.rs");
    std::fs::write(out_path, generated).expect("write bundled extension manifest");
}

fn collect_scheme_files(dir: &Path, output: &mut Vec<PathBuf>) {
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if entry.file_type().is_file()
            && path.extension().is_some_and(|extension| extension == "scm")
            && path.file_name().is_none_or(|name| name != "test.scm")
        {
            output.push(path.to_path_buf());
        }
    }
}
