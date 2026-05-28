use std::path::{Path, PathBuf};

/// Absolute path to the `code/` directory (the Cargo workspace root).
///
/// Determined at compile time from `CARGO_MANIFEST_DIR` — the `utils/` crate
/// lives at `code/utils/`, so `..` is `code/`. All host-side binaries should
/// resolve their `logs/`, `host_storage/`, and other data directories relative
/// to this root, making paths independent of the current working directory.
pub fn workspace_root() -> PathBuf {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).to_path_buf()
}
