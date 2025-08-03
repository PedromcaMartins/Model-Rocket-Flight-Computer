#![allow(dead_code)]
#![deny(unused_must_use)]

use std::{env, path::PathBuf};

use xshell::{cmd, Shell};

fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    match &args[..] {
        ["test", "all"] => test_all(),
        ["test", "host"] => test_host(),
        ["test", "host-target-esp32s3"] => test_host_target_esp32_s3(),
        ["test", "target-esp32s3"] => test_target_esp32_s3(),
        _ => {
            println!("USAGE cargo xtask test [all|host|host-target-esp32s3|target-esp32s3]");
            Ok(())
        }
    }
}

fn test_all() -> Result<(), anyhow::Error> {
    test_host()?;
    test_target_esp32_s3()?;
    test_host_target_esp32_s3()?;

    Ok(())
}

fn test_host() -> Result<(), anyhow::Error> {
    let sh = Shell::new()?;
    sh.change_dir(root_dir());
    cmd!(sh, "cargo test --workspace --exclude host-target-esp32s3-tests --exclude host-target-nucleo-tests").run()?;
    Ok(())
}

fn test_host_target_esp32_s3() -> Result<(), anyhow::Error> {
    flash_esp32_s3()?;

    let sh = Shell::new()?;
    sh.change_dir(root_dir());
    cmd!(sh, "cargo test -p host-target-esp32s3-tests").run()?;

    Ok(())
}

fn test_target_esp32_s3() -> Result<(), anyhow::Error> {
    let sh = Shell::new()?;
    sh.change_dir(root_dir().join("cross-esp32-s3"));
    cmd!(sh, "cargo test -p self-tests").run()?;
    Ok(())
}

fn flash_esp32_s3() -> Result<(), anyhow::Error> {
    let sh = Shell::new()?;
    sh.change_dir(root_dir().join("cross-esp32-s3"));
    cmd!(sh, "cargo run --release").run()?;
    Ok(())
}

fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();
    xtask_dir
}
