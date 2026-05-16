# `code/` — Rust workspace

This directory is a Cargo workspace containing every piece of software in the project, plus the **detailed design** docs that go with each crate. Architecture and interface design live in [`../docs/`](../docs/) — see [`../docs/README.md`](../docs/README.md#architecture--detailed-design) for the split.

## Scope of the docs in this folder

This folder owns **detailed design**: choice of crates, internal module layout, async runtime selection, error-type strategy, test harnesses, build/run recipes, embedded-target specifics, and implementation TODOs. Anything that would change if you reimplemented the same architecture differently belongs here.

It does **not** own goals, non-goals, system constraints, or cross-subsystem interface contracts — those live in `docs/`.

## Workspace members

| Crate | Purpose | Std/no_std |
|---|---|---|
| `flight-computer/` | Hardware-agnostic flight computer library: sensor traits, state machine, deployment logic, telemetry tasks, test utilities. Embedded firmware crates (e.g. `cross-esp32-s3`, `cross-nucleo-f413zh`) consume this. | `no_std` core, `std` test utils simulators |
| `proto/` | Telemetry message definitions, newtypes (units), records, events, errors. The wire-format contract shared between flight computer, simulator, and ground station. | `no_std` |
| `simulator/` | Host-side simulator: physics, scripted scenarios, runtime, API. Drives the flight computer library through its sensor/actuator interfaces for SITL testing. | `std` |
| `flight-computer-host/` | Host-side FC binary — binds two interprocess local sockets (Linux + Windows transparently via `GenericNamespaced`), runs the FC library with simulator-fed peripherals over `fc-sim.sock` and GS telemetry over `fc-gs.sock`. | `std` |
| `ground-station-backend/` | Ground-station server: postcard client to the FC, REST/DB layer for the frontend. Binaries live in `src/bin/`. | `std` |
| `xtask/` | Project task runner (build, run, test orchestration). Invoke via `cargo xtask <task>`. | `std` |

Embedded-target binary crates (`cross-esp32-s3`, `cross-nucleo-f413zh`) are referenced from the root `README.md`; they live outside this workspace because they need different toolchains.

## Where to read more

- Per-crate detailed design lives in each crate's own `README.md` (add one when it doesn't exist yet — see [`../AGENTS.md`](../AGENTS.md) §1).
- Module-level documentation lives in rustdoc (`//!` at the top of `lib.rs` or `mod.rs`). Run `cargo doc --open` from this directory.
- Cross-cutting decisions about *which* crate owns a responsibility live in [`../docs/`](../docs/).

## Build & run

Common commands (run from this directory):

```bash
cargo build                       # build the whole workspace
cargo test                        # run host-side tests
cargo doc --open                  # browse rustdoc
cargo xtask <task>                # project-specific tasks (see xtask/)
```

Embedded targets have their own commands; see the respective `cross-*` crate.

## Important: `no_std` ≠ test constraints

Despite being a `no_std` crate in production, `flight-computer` enables `std` during tests
(`#![cfg_attr(not(any(test, feature = "std")), no_std)]` at `lib.rs:38`). This means:

- **Dev-dependencies do not need to be `no_std`-compatible.** `std` is always available
  in `#[cfg(test)]` code. A comment like "no_std compatible" on a dev-dependency is
  misleading — the real constraint is "avoids unnecessary default features."
- **Test code can freely use `std`** — including `std::thread`, `std::sync`, filesystem I/O, etc.
- **`cfg(test)` gates** are separate from feature gates; `std` is available in test mode
  regardless of which features are selected.

## Patches

The root `Cargo.toml` patches several crates (`postcard-rpc`, `postcard-schema`, `bmp280-ehal`, `switch-hal`) to local sibling checkouts. Those checkouts must exist at `../../<crate>/` for builds to succeed. If a patch is no longer needed, remove it — do not leave dead patches in place.

## TODOs

See the project-wide [TODO policy](../docs/how-we-work.md#todo-policy). Crate-internal TODOs belong in the crate's own `TODO.md`; system-level TODOs go in [`../docs/TODO.md`](../docs/TODO.md).
