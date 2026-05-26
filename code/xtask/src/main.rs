#![allow(dead_code)]
#![deny(unused_must_use)]

mod host;

use std::env;

use anyhow::Result;

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|s| &**s).collect::<Vec<_>>();

    match &args[..] {
        ["host"] => host::run_host(),
        _ => {
            println!("USAGE cargo xtask host");
            Ok(())
        }
    }
}
