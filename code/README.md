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

## Error handling

Host-level crates (`simulator`, `flight-computer-host`, `ground-station-backend`)
follow these conventions:

**1. `panic!()` is for program bugs, not operational failures.**
A desynced IPC pipe, a dropped channel, or a failed subscribe are expected
operational conditions — use `anyhow::Result` + `?` / `bail!`.

**2. `.expect()` is `panic!()` in a trench coat.**
Both abort the process. Replace with:
- `?` — propagate up the call stack
- `.context()?` / `.with_context()?` — attach context then propagate
- `.ok()` / `let _ =` — silently skip non-fatal failures

**3. Function signatures must reflect failure modes.**
If a function can fail, its return type says so:

```rust
// bad
pub async fn run(...) { ... panic!(...); }

// good
pub async fn run(...) -> anyhow::Result<()> { ... }
```

**4. Spawned tasks propagate errors through `JoinHandle`.**
A `tokio::spawn` closure returning `Result<T, anyhow::Error>` makes the
`JoinHandle` carry the error. The parent matches on
`Result<anyhow::Result<T>, JoinError>` in `select!`:

| Pattern | Meaning | Action |
|---|---|---|
| `Ok(Err(e))` | Task completed with error | `inner?` (propagate) |
| `Ok(Ok(()))` | Task completed without error | `bail!` (unexpected) |
| `Err(join_err)` | Task panicked | `bail!("task panicked: {join_err}")` |

**5. Attach context at error origin.**
Convert low-level errors at the source with `.with_context(|| "...")` so
callers get descriptive messages without re-wrapping:

```rust
// At source — every caller gets a useful error
async fn subscribe<T: Topic>(&self) -> anyhow::Result<Subscription<T::Message>> {
    self.client.subscribe_exclusive::<T>(...).await
        .with_context(|| format!("subscribe {} failed", T::PATH))
}
```

Each crate documents its own critical-vs-non-critical channel classification in
its detailed-design spec (see `simulator/spec.md` §9 for an example).

### Deferred cleanup

Use `scopeguard::defer!` for any resource that needs cleanup on function exit
(early return, `break`, or normal end). Place the `defer!` immediately after the
corresponding resource acquisition — guards fire in reverse declaration order.

```rust
use scopeguard::defer;

fn example() {
    let Ok(resource) = acquire() else { return };
    defer! { release(&resource); }
    // ... no manual cleanup needed on any exit path
}
```

Only use a custom RAII struct (`impl Drop`) when the guard carries state more
complex than a single closure. Otherwise `defer!` keeps cleanup colocated with
acquisition and eliminates error-path boilerplate.

## Patches

The root `Cargo.toml` patches several crates (`postcard-rpc`, `postcard-schema`, `bmp280-ehal`, `switch-hal`) to local sibling checkouts. Those checkouts must exist at `../../<crate>/` for builds to succeed. If a patch is no longer needed, remove it — do not leave dead patches in place.

## TODOs

See the project-wide [TODO policy](../docs/how-we-work.md#todo-policy). Crate-internal TODOs belong in the crate's own `TODO.md`; system-level TODOs go in [`../docs/TODO.md`](../docs/TODO.md).
