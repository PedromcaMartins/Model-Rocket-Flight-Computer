# Rust toolchain

A `rust-toolchain.toml` should be added to pin these — until then, verify you have the items below.

## Required — won't build/flash/debug without these

| Tool | Role | Kind |
|---|---|---|
| `cargo-binutils` | `cargo-size`, `cargo-objcopy`, `rust-objdump` | `required`, `embedded`, `profiling` |
| `defmt-print` | Decode defmt log frames from RTT output | `required`, `embedded`, `logging` |
| `esp` toolchain | ESP-IDF builds | `required`, `embedded` |
| `espflash` / `cargo-espflash` | Flash ESP32 targets | `required`, `embedded`, `debug` |
| `espup` | Install/update the `esp` toolchain | `required`, `embedded` |
| `flip-link` | Stack-overflow protection linker for Cortex-M | `required`, `embedded` |
| `ldproxy` | Linker proxy for ESP-IDF | `required`, `embedded` |
| `probe-rs` | Flash + RTT debug for ARM targets | `required`, `embedded`, `debug` |
| RISC-V targets (`riscv32imc*`, `riscv32imac*`, `riscv32imafc*`) | ESP32 cross-compile | `required`, `embedded` |
| `thumbv7em-none-eabihf` target | Cortex-M4F cross-compile (Nucleo F413ZH) | `required`, `embedded` |

## QoL — useful for day-to-day work on this repo

| Tool | Role | Kind |
|---|---|---|
| `bacon` | Background check/test watcher | `qol`, `host`, `testing` |
| `bpftrace` | Linux eBPF kernel tracing (ground-station perf debugging, Linux-only) | `qol`, `host`, `profiling` |
| `cargo-bloat` | Binary size analysis (embedded) | `qol`, `host`, `profiling` |
| `cargo-call-stack` | Static call graph + stack usage analysis (embedded) | `qol`, `host`, `profiling` |
| `cargo-criterion` | Benchmark runner for criterion.rs | `qol`, `host`, `testing` |
| `cargo-deny` | License + dependency vetting | `qol`, `host`, `testing` |
| `cargo-embassy` | Embassy codegen helper | `qol`, `host`, `embedded` |
| `cargo-expand` | Expand proc macros (useful for postcard-rpc derives) | `qol`, `host`, `debug` |
| `cargo-hakari` | Workspace-hack dependency management | `qol`, `host` |
| `cargo-llvm-cov` | Code coverage (paired with `cargo-nextest`) | `qol`, `host`, `testing` |
| `cargo-nextest` | Faster, structured test runner | `qol`, `host`, `testing` |
| `cargo-show-asm` | Display assembly/LLVM-IR/MIR for Rust functions | `qol`, `host`, `profiling` |
| `cargo-udeps` | Detect unused dependencies | `qol`, `host`, `testing` |
| `comchan` | Blazingly fast serial monitor with TUI plotting — debugging/logging alternative to `probe-rs` terminal mode | `qol`, `host`, `debug`, `testing` |
| `dioxus-cli` (`dx`) | Dioxus hot-reload build tool for ground-station web frontend | `qol`, `host` |
| `hyperfine` | Command/benchmark timing | `qol`, `host`, `profiling` |
| `inferno` | Flamegraph generation from perf data | `qol`, `host`, `profiling` |
| `just` | Command runner (recipe-based alternative to Make) | `qol`, `host` |
| `savepoint` | Auto-commit when tests pass — TDD workflow assistant | `qol`, `host`, `testing` |
| `sccache` | Compilation cache | `qol`, `host` |
| `scrcpy` | Android screen mirroring for ground-station frontend testing on mobile devices | `qol`, `host`, `testing` |
| `tokio-console` / `dial9` | Async task introspection (ground-station-backend); see also dial9-tokio-telemetry | `qol`, `host`, `debug` |

## Agent-friendly — general-purpose tools also installed

| Tool | Role | Kind |
|---|---|---|
| `bat` | `cat` with syntax-highlighted paging | `host`, `qol` |
| `broot` | Directory tree viewer with fuzzy search | `host`, `qol` |
| `cargo-binstall` | Download cargo binaries (no build) | `host`, `qol` |
| `cargo-cache` | Inspect/clean cargo cache | `host`, `qol` |
| `cargo-generate` | Project scaffolding from templates | `host`, `qol` |
| `cargo-info` | Crate info from the terminal | `host`, `qol` |
| `cargo-update` | Update installed cargo tools | `host`, `qol` |
| `coreutils` | GNU coreutils for Windows | `host`, `qol` |
| `esp-generate` | ESP32 project generation helper | `host`, `embedded` |
| `eza` | Modern `ls` replacement | `host`, `qol` |
| `rg` (ripgrep) | Fast recursive grep | `host`, `qol` |
| `zoxide` | Smarter `cd` with fuzzy learning | `host`, `qol` |

Also installed and may come in handy: `trunk` (Wasm web dev), `wasm-pack` (Wasm packaging), `sqlx-cli` (SQLite/Postgres migrations).

## Notable crates — library dependencies worth knowing about

These aren't CLI tools — they appear in `Cargo.toml` as dependencies. Some are active, others are candidates listed for discoverability.

| Crate / Workspace Crate | Why it's included | Kind |
|---|---|---|
| `anyhow` | Flexible error handling with context (in xtask) | `host`, `crates` |
| `bmp280-ehal` | BMP280 pressure/temperature sensor driver (patched, local crate) | `embedded`, `crates` |
| `bno055` | BNO055 IMU driver (orientation, accel, gyro) | `embedded`, `crates` |
| `bytemuck` | Zero-cost cast between repr(C) types (embedded packet handling) | `embedded`, `crates` |
| `cfg-if` | Ergonomic `#[cfg]` cascading macro — no_std | `embedded`, `no_std`, `crates` |
| `chrono` | Date/time library | `host`, `crates` |
| `circular-buffer` | Fixed-size circular buffer | `host`, `embedded`, `crates` |
| `clap` | CLI argument parser with derive + env support | `host`, `crates` |
| `cobs` / `slip` | Serial framing protocols (COBS and SLIP) for link-layer packet boundaries | `embedded`, `host`, `crates` |
| `color-eyre` | Colored error reporting hook | `host`, `crates` |
| `console-subscriber` | Tokio-console subscriber for async task introspection | `host`, `debug`, `crates` |
| `cortex-m` / `cortex-m-rt` | Cortex-M peripheral access + startup/vector table | `embedded`, `crates` |
| `crc` | CRC checksum computation (telemetry packet integrity) | `embedded`, `no_std`, `crates` |
| `criterion` | Benchmarking framework (paired with `cargo-criterion`) | `testing`, `host`, `crates` |
| `crossbeam` | Concurrent channels, scoped threads, epoch GC — std only | `host`, `crates` |
| `defmt` | Efficient logging framework for embedded (no_std) | `embedded`, `logging`, `crates` |
| `defmt-decoder` | Decode defmt log frames from RTT output | `host`, `embedded`, `logging`, `crates` |
| `defmt-or-log` | Dual-purpose logging: defmt on device, log on host — no_std | `embedded`, `logging`, `crates` |
| `defmt-parser` | Parse defmt log stream | `host`, `embedded`, `logging`, `crates` |
| `derive_more` | Derive macros (`From`, `Into`, `Deref`, `Display`) — used in proto + GS backend | `no_std`, `crates` |
| `device-driver` | Framework for writing embedded device drivers | `embedded`, `crates` |
| `dial9-tokio-telemetry` | Tokio runtime flight recorder — async debugging alternative to `tokio-console` | `host`, `debug`, `crates` |
| `eframe` / `egui` / `egui_plot` / `egui_extras` | Native GUI framework + widgets + real-time plots (ground-station UI) | `host`, `crates` |
| `egui-plotter` | Bridge between egui and plotters backend | `host`, `crates` |
| `embassy-executor` | Async executor for embedded (Cortex-M, RISC-V, x86) | `embedded`, `crates` |
| `embassy-futures` | Async utilities for embassy | `embedded`, `crates` |
| `embassy-sync` | Async primitives (channels, mutex, pipe) for embedded — FC task communication | `embedded`, `crates` |
| `embassy-time` | Async timers, delays, time drivers for embedded | `embedded`, `crates` |
| `embedded-cli` | CLI with autocompletion for embedded (`no_std`) — debug REPL on FC | `embedded`, `crates` |
| `embedded-hal` / `embedded-hal-async` | Hardware abstraction layer traits (blocking + async) | `embedded`, `crates` |
| `embedded-hal-bus` | Bus sharing (SPI, I2C) for embedded | `embedded`, `crates` |
| `embedded-hal-fuzz` | Fuzz embedded-hal implementations | `embedded`, `testing`, `crates` |
| `embedded-hal-mock` | Mock embedded-hal implementations for unit testing | `embedded`, `testing`, `crates` |
| `embedded-io` / `embedded-io-async` | I/O traits for embedded | `embedded`, `crates` |
| `embedded-io-adapters` | Adapters between embedded-io and tokio/futures | `embedded`, `host`, `crates` |
| `embedded-sdmmc` | SD/MMC card driver for embedded (data logging) | `embedded`, `no_std`, `crates` |
| `embedded-storage` | Storage traits for embedded (flash, etc.) | `embedded`, `crates` |
| `embedded-test` | `no_std` test harness for device-side tests | `embedded`, `testing`, `crates` |
| `embedded-update` | Firmware update protocol | `embedded`, `crates` |
| `env_logger` | Environment-variable-configured logger | `host`, `logging`, `crates` |
| `heapless` | Heap-less data structures (`Vec`, `String`, `HashMap`) with serde | `embedded`, `no_std`, `crates` |
| `hayasen` | Unified embedded sensor library (MPU9250, MPU6050, MAX30102) — potential IMU driver alternative to `bno055` | `embedded`, `crates` |
| `hifitime` | High-fidelity time (TAI/UTC/ET/TDB) — no_std, leap-second aware | `embedded`, `no_std`, `crates` |
| `interprocess` | Cross-platform local sockets/IPC (tokio) — host IPC adapter | `host`, `crates` |
| `log` | Lightweight logging facade | `embedded`, `host`, `logging`, `crates` |
| `miette` | Fancy diagnostic error reporting with suggestions | `host`, `crates` |
| `mockall` | Mock object library for trait-based testing | `testing`, `host`, `crates` |
| `nalgebra` | Linear algebra library (no_std + serde) | `no_std`, `crates` |
| `nmea` | NMEA 0183 GPS sentence parser with defmt + serde | `embedded`, `no_std`, `crates` |
| `notify` | Cross-platform file system notifications | `host`, `crates` |
| `num_enum` | Enum/primitive conversion (`IntoPrimitive`, `TryFromPrimitive`) — no_std | `embedded`, `no_std`, `crates` |
| `panic-halt` | Halts on panic (no output) — embedded panic handler | `embedded`, `crates` |
| `panic-probe` | Probe-based panic handler with defmt output | `embedded`, `logging`, `crates` |
| `parking_lot` | Faster mutexes/rwlocks/condvars than `std::sync` — ground-station concurrency | `host`, `crates` |
| `plotters` / `plotters-backend` | Plotting library with egui integration | `host`, `crates` |
| `postcard` | Compact binary serialization format | `embedded`, `no_std`, `crates` |
| `postcard-rpc` | RPC framework over postcard — main IPC between FC and ground station | `embedded`, `no_std`, `crates` |
| `postcard-schema` | Schema generation for postcard types with nalgebra/uom/chrono support | `embedded`, `no_std`, `crates` |
| `rand` | Random number generation (testing, sensor noise simulation) | `host`, `embedded`, `crates` |
| `ratatui` | TUI framework for terminal dashboards — ground-station monitoring | `host`, `crates` |
| `rayon` | Data-parallelism library for CPU-bound work — ground-station data processing | `host`, `crates` |
| `rocket` | Web framework for ground-station REST API | `host`, `crates` |
| `serde` | Serialization framework (core dependency, used across all crates) | `embedded`, `host`, `no_std`, `crates` |
| `serde-json-core` | no_std JSON serializer/deserializer | `embedded`, `no_std`, `crates` |
| `sguaba` | Hard-to-misuse rigid body transforms (WGS84/ECEF/NED/FRD/ENU) — coordinate math for rocketry sensor fusion | `embedded`, `no_std`, `crates` |
| `static_cell` | Statically allocated cells for `&'static mut` | `embedded`, `crates` |
| `switch-hal` | Switch/button HAL (patched, local crate) | `embedded`, `crates` |
| `thiserror` | `derive(Error)` for error enums (in flight-computer) | `no_std`, `crates` |
| `time` | Date/time library (low-dependency alternative to chrono) | `host`, `embedded`, `crates` |
| `tokio` | Async runtime (ground-station backend) | `host`, `crates` |
| `tokio-serial` | Serial port I/O for tokio | `host`, `crates` |
| `tracing` | Structured diagnostics/telemetry (ground-station backend) | `host`, `logging`, `crates` |
| `tracing-appender` | Non-blocking log file writer for tracing | `host`, `logging`, `crates` |
| `tracing-flame` | Flamegraph generation from tracing span data | `host`, `profiling`, `crates` |
| `tracing-log` | Bridge between `log` facade and `tracing` | `host`, `logging`, `crates` |
| `tracing-subscriber` | Subscriber with fmt/env-filter/json output (ground-station logging config) | `host`, `logging`, `crates` |
| `uom` | Type-safe units of measure (SI with serde + defmt) | `no_std`, `crates` |
| `xshell` | Shell command helpers for xtask build scripts | `host`, `crates` |

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
