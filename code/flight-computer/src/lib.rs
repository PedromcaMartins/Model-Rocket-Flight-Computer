//! Flight computer library — hardware-agnostic FC logic.
//!
//! Contains the FSM, sensor tasks, deployment logic, telemetry, and postcard-rpc
//! server. Peripheral implementations are selected at compile time via feature
//! flags; the library itself is runtime-neutral and never spawns tasks or
//! creates executors.
//!
//! See `docs/software/spec.md §6` for the architectural constraints.
//!
//! # Features
//!
//! | Feature | What it enables |
//! |---|---|
//! | `impl_embedded` | Real hardware drivers (`embedded-hal`) — used in HW firmware |
//! | `impl_sim` | Simulator-fed postcard-rpc peripheral clients — transport-agnostic; used in SIL (HOST) and PIL |
//! | `impl_host` | `HostFileSystem` over a host directory — orthogonal to `impl_sim`; used in the HOST binary |
//! | `host` | Convenience alias: `impl_sim` + `impl_host` + `log` + `proto/host` — everything a HOST binary needs |
//! | `std` | Standard library (required by `impl_sim` and `impl_host`) |
//! | `log` | Logging via the `log` crate (default for host/test builds) |
//! | `defmt` | Logging via `defmt` (for embedded targets) |
//!
//! `impl_embedded` and `impl_sim` are mutually exclusive at link time.
//! `impl_host` (filesystem) composes independently with either.
//!
//! # Verification
//!
//! ```bash
//! # HW: real hardware drivers
//! cargo clippy --no-default-features --features impl_embedded,defmt -p flight-computer
//!
//! # SIL / PIL: simulator peripheral clients
//! cargo clippy --no-default-features --features impl_sim,log -p flight-computer
//!
//! # HOST binary combination
//! cargo clippy --no-default-features --features impl_sim,impl_host,log -p flight-computer
//! ```

#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(unsafe_code)]
#![deny(unused_must_use)]

#![allow(async_fn_in_trait)]

// Crate used for single-threaded
#![allow(clippy::future_not_send)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::unused_async)]
#![allow(clippy::uninlined_format_args)]

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub(crate) mod core;
pub(crate) mod config;
pub(crate) mod log;
pub(crate) mod sync;

pub mod interfaces;
pub mod tasks;

#[cfg(test)]
pub mod test_utils;
