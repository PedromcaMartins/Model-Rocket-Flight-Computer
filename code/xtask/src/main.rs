#![deny(unused_must_use)]

mod check;
mod host;

use std::env;

use anyhow::Result;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    match &args[..] {
        ["host"] => host::run_host(),
        ["check"] => check::run_check(),
        _ => {
            println!("USAGE: cargo xtask <command>");
            println!();
            println!("Commands:");
            println!("  check    Clippy → build (dev+release) → test (dev+release) on the");
            println!("           whole workspace (host features only)");
            println!("  host     Build and run the full host stack (FC + GS + Simulator)");
            Ok(())
        }
    }
}
