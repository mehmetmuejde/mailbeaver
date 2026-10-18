//! Embeds the built UI (`dist/` from `pnpm build`) into the binary, so the server is one
//! self-contained file. Without a build the table is empty and the server says so.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);
    let dist = manifest_dir.join("../../dist");
    // Cargo scans a directory recursively for changes.
    println!("cargo:rerun-if-changed={}", dist.display());

    let mut files = Vec::new();
    if dist.is_dir() {
        collect(&dist, &dist, &mut files)?;
    }
    files.sort();

    let mut code = String::from("pub static ASSETS: &[(&str, &[u8])] = &[\n");
    for (path, absolute) in &files {
        code.push_str(&format!("    ({path:?}, include_bytes!({absolute:?})),\n"));
    }
    code.push_str("];\n");
    fs::write(PathBuf::from(var("OUT_DIR")?).join("assets.rs"), code)
}

fn var(name: &str) -> io::Result<String> {
    env::var(name).map_err(|err| io::Error::other(format!("{name}: {err}")))
}

/// Every file below `dir` as (path relative to `root` with `/`, absolute path).
fn collect(root: &Path, dir: &Path, files: &mut Vec<(String, String)>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, files)?;
        } else if let Ok(relative) = path.strip_prefix(root) {
            let relative = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let absolute = path.canonicalize()?.to_string_lossy().into_owned();
            files.push((relative, absolute));
        }
    }
    Ok(())
}
