# Rust toolchain

A `rust-toolchain.toml` should be added to pin these — until then, verify you have the items below.

## Required — won't build/flash/debug without these

| Tool | Role |
|---|---|
| `thumbv7em-none-eabihf` target | Cortex-M4F cross-compile (Nucleo F413ZH) |
| RISC-V targets (`riscv32imc*`, `riscv32imac*`, `riscv32imafc*`) | ESP32 cross-compile |
| `esp` toolchain | ESP-IDF builds |
| `flip-link` | Stack-overflow protection linker for Cortex-M |
| `ldproxy` | Linker proxy for ESP-IDF |
| `probe-rs` | Flash + RTT debug for ARM targets |
| `espflash` / `cargo-espflash` | Flash ESP32 targets |
| `espup` | Install/update the `esp` toolchain |
| `defmt-print` | Decode defmt log frames from RTT output |
| `cargo-binutils` | `cargo-size`, `cargo-objcopy`, `rust-objdump` — binary inspection |

## QoL — useful for day-to-day work on this repo

| Tool | Role |
|---|---|
| `cargo-nextest` | Faster, structured test runner |
| `bacon` | Background check/test watcher |
| `tokio-console` | Async task introspection (ground-station-backend) |
| `cargo-expand` | Expand proc macros (useful for postcard-rpc derives) |
| `cargo-bloat` | Binary size analysis (embedded) |
| `cargo-call-stack` | Static call graph + stack usage analysis (embedded) |
| `cargo-embassy` | Embassy codegen helper |
| `sccache` | Compilation cache |
| `inferno` | Flamegraph generation from perf data |
| `hyperfine` | Command/benchmark timing |

## Agent-friendly — general-purpose tools also installed

| Tool | Role |
|---|---|
| `bat` | `cat` with syntax-highlighted paging |
| `eza` | Modern `ls` replacement |
| `rg` (ripgrep) | Fast recursive grep |
| `zoxide` | Smarter `cd` with fuzzy learning |
| `cargo-cache` | Inspect/clean cargo cache |
| `coreutils` | GNU coreutils for Windows |
| `cargo-binstall` | Download cargo binaries (no build) |
| `cargo-update` | Update installed cargo tools |
| `cargo-generate` | Project scaffolding from templates |
| `cargo-info` | Crate info from the terminal |
| `esp-generate` | ESP32 project generation helper |

Also installed and may come in handy: `trunk` (Wasm web dev), `wasm-pack` (Wasm packaging), `sqlx-cli` (SQLite/Postgres migrations).

## Versions (as of 2026-05-08)

- **Rust (stable):** `rustc 1.86-x86_64-pc-windows-msvc`
- **Rust (nightly):** `nightly-x86_64-pc-windows-msvc`
- **ESP toolchain:** `esp` (installed via espup)
- **cargo-nextest:** `0.9.133`
- **probe-rs:** recent (via cargo)
- **espflash / cargo-espflash:** `4.3.0`
- **flip-link:** `0.1.12`
- **ldproxy:** `0.3.4`
- **bacon:** `3.22.0`
- **sccache:** `0.14.0`
- **cargo-binutils:** `0.4.0`
- **defmt-print:** `1.0.0`
- **cargo-expand:** `1.0.121`
- **cargo-bloat:** `0.12.1`
- **cargo-call-stack:** `0.1.16`
- **cargo-embassy:** `0.3.6`
- **tokio-console:** `0.1.14`
- **inferno:** `0.12.6`
- **hyperfine:** `1.20.0`
- **bat:** `0.26.1`
- **eza:** `0.23.4`
- **ripgrep:** `15.1.0`
- **zoxide:** `0.9.9`
- **cargo-cache:** `0.8.3`
- **coreutils:** `0.8.0`
- **cargo-binstall:** `1.18.1`
- **cargo-update:** `20.0.0`
- **cargo-generate:** `0.23.8`
- **cargo-info:** `0.7.7`
- **esp-generate:** `1.2.0`
- **trunk:** `0.21.14`
- **wasm-pack:** `0.14.0`
- **sqlx-cli:** `0.7.4`
