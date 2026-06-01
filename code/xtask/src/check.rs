use std::path::PathBuf;

use anyhow::Result;
use xshell::{cmd, Shell};

/// Run clippy, build, and test across the whole workspace using only
/// host-compatible (default) features.  Embedded-specific features
/// (defmt, embassy-time, etc.) are tested through their respective
/// binary crates.
///
/// The xtask crate is excluded from build/test because it owns the
/// running process (Windows won't overwrite a locked binary).
pub fn run_check() -> Result<()> {
    let sh = Shell::new()?;
    sh.change_dir(root_dir());

    let exclude = ["--exclude", "xtask"];

    eprintln!("=== Step 1/5: Clippy ===");
    cmd!(sh, "cargo clippy --workspace --all-targets").run()?;

    // eprintln!("=== Step 2/5: Build (dev profile) ===");
    // cmd!(sh, "cargo build --workspace {exclude...}").run()?;

    // eprintln!("=== Step 3/5: Build (release profile) ===");
    // cmd!(sh, "cargo build --workspace --release {exclude...}").run()?;

    eprintln!("=== Step 4/5: Test (dev profile) ===");
    cmd!(sh, "cargo nextest run --workspace {exclude...}").run()?;

    // eprintln!("=== Step 5/5: Test (release profile) ===");
    // cmd!(sh, "cargo nextest run --workspace --release {exclude...}")
    //     .run()?;

    eprintln!("=== All checks passed! ===");
    Ok(())
}

fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();
    xtask_dir
}
