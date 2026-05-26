# Host-Stack Roadmap

Tracks the work to split the monolithic HOST binary into four independent processes (FC, simulator, GS backend, GS frontend) connected by postcard-rpc, as decided in [ADR-001](ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md).

The target architecture is specified in [`docs/software/spec.md`](docs/software/spec.md). This roadmap tracks the path from the current monolithic state to that architecture across three milestones.

Each task still requires a written artifact before implementation begins: an **ADR** for structural/design decisions, a **Spec** for full subsystem work.

**Artifact convention.** follow [`AGENTS.md`](AGENTS.md) with a split between *architectural constraints* (which go in the spec) and *detailed design* (which goes in the crate).

---

## Architecture overview

### Current state (baseline)

A single `HOST` binary wires the FC, simulator, and GS together in-process using Embassy signals and shared memory. There are no process boundaries, no postcard-rpc on host, and no independent GS processes.

```
HOST binary (today)
===================

  ┌──────────────────────────────────────────────────────┐
  │ monolithic host binary                               │
  │                                                      │
  │   FC tasks ←─── in-process signals ───→ simulator    │
  │                                                      │
  │   GS (partial, broken) ←─── direct fn calls          │
  └──────────────────────────────────────────────────────┘
```

### Target state (spec.md)

Four independent processes communicating exclusively over postcard-rpc sockets, orchestrated by `xtask`. The FC library compiles unchanged for embedded and host targets; only the peripheral implementation differs.

```
host machine (HOST mode — target)
==================================

                      ( operator )
                            │
                            ▼
      ┌───────────────────────────────────────┐
      │ ground-station-frontend (ratatui TUI) │
      └────────────────────┬──────────────────┘
                           │ REST / JSON
                           ▼
      ┌───────────────────────────────────────┐
      │ ground-station-backend                │
      │ REST + storage + config               │ ◄─── ( scenario config file )
      └──────┬──────────────────────────┬─────┘
             │                          │
       fc-gs.sock                 sim-gs.sock
       postcard-rpc               postcard-rpc
       telemetry / commands       lifecycle / triggers / status / hash
             │                          │
             ▼                          ▼
  ┌──────────────────────┐    ┌─────────────────────────────────┐
  │ flight-computer-host │    │ simulator                       │
  │ (FC lib + impl_sim   │    │ (physics + events + TUI)        │
  │   + impl_host FS)    │    └────────────────┬────────────────┘
  └──────────┬───────────┘                     │
             ▲                                 │
             │       fc-sim.sock               │
             │       postcard-rpc              │
             └────── sensors / arming / ───────┘
                     deploy / LED
```

---

## Milestone 1 — Wire vocabulary and FC library (proto + flight-computer cleanup)

**Goal:** `proto` carries the full shared wire vocabulary with correct feature gating; the FC library compiles cleanly for all three deployment targets (`impl_embedded`, `impl_sim`, `impl_host`) without the monolithic host wiring.

Nothing runs end-to-end yet. This milestone is purely about getting the shared contract and the library right before building binaries against them.

### M1.1 — Proto feature gating

Gate `proto`'s contents so embedded targets never compile host-only symbols, and host targets can opt into the IPC adapter.

**Architectural constraint** (in `docs/software/spec.md §9`): `proto` stays `#![no_std]`; embedded targets never see sim-only or IPC symbols; IPC deps (`tokio`, `interprocess`) are opt-in. **Detailed design** (in `code/proto/`): exact feature flag names, `#[cfg]` on `topics!`/`endpoints!` blocks, `InterprocessWireTx`/`Rx` type signatures.

| Feature | Contents |
|---|---|
| `default` (HW-safe) | `PingEndpoint`, `GlobalTickHzEndpoint`, `RecordTopic` |
| `simulator-endpoints` | All `Sim*` topics (altimeter, GPS, IMU, arm, deployment, LEDs) |
| `ipc-adapter` | `InterprocessWireTx` / `InterprocessWireRx` over `interprocess` + `tokio` |
| `host` | `simulator-endpoints` + `ipc-adapter` |
| `pil` | `simulator-endpoints` |

**Status:** Not started.

### M1.2 — FC library cleanup: `impl_software` → `impl_sim` rename

Restructure `flight-computer`'s feature flags to reflect what has actually changed since the original split: the transport layer, not the peripheral model.

**Architectural constraint** (in `docs/software/spec.md §10`): peripheral feature flags are mutually exclusive; `impl_host` is orthogonal and composes with `impl_sim`. **Detailed design** (in `code/flight-computer/`): exact feature names, `#[cfg]` on impl modules, `start_*` entry point signatures.

#### Peripheral feature flags — rename and clarification

Currently the codebase uses `impl_software`. This is being renamed to `impl_sim` to accurately reflect what the feature contains. Both SIL (HOST mode) and PIL use these same postcard-rpc sim peripheral clients; the only difference between them is the transport underneath (interprocess socket vs USB serial), which is supplied by the calling binary — not by this feature.

| Feature | Current name | New name | What it enables | Used by |
|---|---|---|---|---|
| Embedded HW drivers | `impl_embedded` | `impl_embedded` (unchanged) | Real hardware peripherals: I2C altimeter, SPI IMU, SPI GPS, SPI filesystem, GPIO arming/deployment | HW firmware |
| Sim peripheral clients | `impl_software` | **`impl_sim`** (rename) | postcard-rpc sim peripheral clients — transport-agnostic (`SimAltimeter`, `SimGps`, `SimImu`, `SimArming`, `SimRecovery`, `SimLed*`) | SIL (HOST mode) and PIL |
| Host filesystem | `impl_host` | `impl_host` (unchanged) | `HostFileSystem` — implements the `FileSystem` trait over a directory on the host machine | HOST binary only; PIL and HW use SPI flash/SD |

`impl_embedded` and `impl_sim` are mutually exclusive peripheral feature flags. `impl_host` (filesystem) is orthogonal and composes with `impl_sim` in the HOST binary. Default features drop to `["log", "std"]`; consumers opt in explicitly.

Verification gates: `cargo check` must pass independently for:
- `--no-default-features --features impl_embedded` (no_std, HW)
- `--no-default-features --features impl_sim` (std, PIL)
- `--no-default-features --features impl_sim,impl_host` (std, HOST binary combination)

**Status:** Done.

### M1.3 — Task lifecycle separation: `run_flight_computer` + cooperative storage exit

Extract the `select`/`join` composition from `start_pil_flight_computer` into a generic `run_flight_computer` function that takes pre-created task futures. This separates future creation from execution.

Modify `storage_task` to observe `FlightState::Touchdown` via `FLIGHT_STATE_WATCH` and exit cleanly after a configurable hold-timer, enabling the `select(join(fsm, storage), join5(...))` orchestration shape defined in `spec.md §6.6`.

The `start_pil_flight_computer` function becomes a pure factory — creates all 7 task futures and passes them to `run_flight_computer`. The `start_sil_flight_computer` entry point is unchanged.

**Status:** Done.

---

## Milestone 2 — Independent binaries (FC-host + Simulator)

**Goal:** `flight-computer-host` and `simulator` run as separate processes connected by `fc-sim.sock`. A full HOST scenario (sensor data → FC FSM → deployment) can execute end-to-end without any GS process present.

This is the first point at which the architecture matches the spec for the FC ↔ Simulator boundary.

### M2.1 — `flight-computer-host` binary

New workspace crate `code/flight-computer-host/` (native toolchain, inside workspace).

Architectural role:
- Links the FC library with `impl_host` peripherals.
- Connects to the simulator's postcard-rpc server via `fc-sim.sock`.
- Starts FC task set: sensor tasks, FSM, storage task, telemetry relay.
- Owns tracing initialisation (console + file sink).
- GS connection via `fc-gs.sock` (see M3).

**Status:** Done.

### M2.2 — Simulator binary

Standalone binary in `code/simulator/`.

Architectural role:
- Physics tick loop publishes sensor Topics as postcard-rpc client on `fc-sim.sock`.
- FC deployment/LED calls arrive as Endpoint requests from FC's server.
- Scripted scenario with timed trigger events from compile-time config struct.
- Minimal read-only TUI: live physics state (altitude, velocity, acceleration, sim time), actuator state (LEDs, deployment), config summary.
- tracing initialisation (console + file sink), panic hook.

Out of scope for M2.2:
- GS interaction (`sim-gs.sock`, lifecycle, config handshake) — see M2.4+ / M3.2+.
- Two-phase lifecycle — binary runs physics on launch; Ctrl-C to exit.
- Config from file — values are in a Rust `pub const` struct; recompile to change.
- Interactive TUI controls — read-only display only.

Physics scope: parabolic 1D trajectory (motor burn → coast → apogee → descent). Sufficient to exercise the full FC FSM.

**Status:** Done.

### M2.3 — Simulator 3D physics (spatial crate + kinematic attitude)

Enhance the physics engine from 1D to 3D using the new `code/spatial/` crate
for type-safe coordinate frame conversions. Frame conversions run on the host
only — the FC receives sensor data in its native wire format.

**New crate:** `code/spatial/` — wraps sguaba, defines `LaunchNed` and `RocketFrd`
frames, provides conversion functions between spatial types and proto wire types.
See `code/simulator/spec.md §2.5`. Depends on nalgebra 0.34 + uom 0.38 (alongside
proto's nalgebra 0.33 + uom 0.37).

**3D physics scope:**
- Launch position, inclination and azimuth.
- Multi-axis force composition: gravity (NED), thrust (RocketFrd → NED via attitude),
  drag (RocketFrd → NED), recovery (NED, opposing velocity).
- **Kinematic attitude** — nose aligns with velocity direction via
  `UnitQuaternion::rotation_between`. No rotational dynamics.
- 3D position/velocity/acceleration state in `LaunchNed` frame.
- Attitude (yaw/pitch/roll) for TUI display and IMU output.
- Ground contact check uses NED vertical coordinate.

**Sensor output changes (PhysicsState → Proto conversions):**
- GPS: NED position → ECEF → WGS84 via `FrameConversions::ned_to_gps`.
- IMU: body-frame acceleration (rotated from NED) + angular velocity.
- Altimeter: altitude = launchpad MSL − NED down component.

**TUI update:** Physics panel adds downrange distance, bearing, velocity magnitude,
attitude (yaw/pitch/roll).

**Status:** Not started.

### Full 6-DOF rotational dynamics (deferred)

Replace kinematic attitude with full rotational dynamics:

Scope:
- Rocket moment of inertia (MOI) about each axis.
- Aerodynamic moments: center-of-pressure vs. center-of-gravity offset.
- Thrust misalignment torque during motor burn.
- Angular acceleration → angular velocity → attitude integration.
- Fin-based passive stability model (restoring moment proportional to angle of attack).

Config additions: MOI tensor, CP location, CG location, fin geometry.

**Status:** Deferred — building on spatial crate from M2.3.

### M2.4 — Simulator config, lifecycle & interactive TUI

Add production features to the simulator binary.

Scope:
- Config loading from TOML file (replacing compile-time struct), validation, hashing.
- Full interactive TUI with manual trigger controls (ignition, arm, deploy) and lifecycle controls.
- Internal lifecycle (setup → runtime) with Restart support (re-read config, reset physics).
- GS connectivity deferred to M3.2+ — the TUI provides the operator interface until then.

**Status:** Deferred.

---

## Milestone 3 — Ground station (GS backend + GS frontend)

**Goal:** The full four-process HOST topology from spec.md is running. An operator can start a scenario, observe live telemetry, issue commands, and see crash/disconnect state correctly reflected in the UI.

### M3.1 — GS backend: REST API + storage (FC-facing)

Total rewrite crate `code/ground-station-backend/`.

Architectural role:
- Connects to FC on `fc-gs.sock` (telemetry subscriber, command issuer).
- Stores FC telemetry records to disk (append-only, one file per session).
- Exposes REST/JSON API consumed exclusively by the frontend — GS frontend never speaks postcard-rpc.
- Independent of the simulator for its core function: operates on FC telemetry alone.

Cleanup before building:
- Remove outdated code `code/ground-station-backend/` — its `PostcardClient` and `postcard_server` are superseded by `proto::PostcardClient` + `proto::transport::thread`.
- `PostcardClient::try_new_raw_nusb` (USB serial) needs a new home if still needed.

Simulator integration deferrred:
- `sim-gs.sock` connection (lifecycle, config-hash, manual triggers) postponed to M3.3+.

Config convention cleanup (see `AGENTS.md §6`):
- Convert `RESTApiConfig`, `LocalPostcardConfig` to unit structs with `pub const` values.
- Extract `LoggingConfig` constant fields into a separate unit struct; keep only runtime-timestamp fields in `LoggingConfig`.
- Update call sites.

**Status:** Blocked — requires M2 running.

### M3.2 — GS frontend TUI

New ratatui binary.

Architectural role:
- Pure REST client of GS backend; no direct postcard-rpc.
- Minimum viable screens: live telemetry (altimeter, GPS, IMU, flight state), log tail, manual controls (arm, ignition, deploy).
- Disconnect UX: affected panel turns red within one UI refresh; last-known state stays visible, dimmed, with stale-indicator badge; Restart / Shutdown buttons available; no automatic reconnect retry.

**Status:** Blocked — requires M3.1 REST contract settled.

### M3.3 — Simulator-GS integration

Wire the simulator into the GS topology over `sim-gs.sock`.

Phased approach:
1. **State data** — simulator publishes status Topics (physics state, active events, actuator states, config summary) to GS backend.
2. **Config ownership** — GS backend becomes source of truth for scenario config; simulator loads from file path, both compare hashes.
3. **Lifecycle & triggers** — GS controls simulator lifecycle (Start / Restart / Shutdown) and forwards manual triggers (arm, ignition, deploy) over `sim-gs.sock`.

**Status:** Blocked — requires M2.4 (simulator ready for GS interaction) and M3.1 settling the REST/frontend contract.

---

## Milestone 4 — Orchestration (`xtask run-host`)

**Goal:** A single command starts the full HOST stack in dependency order, with correct process supervision and crash policy enforced.

Architectural role of `xtask` in HOST:
- Spawns processes in order: GS backend → Simulator → FC-host → GS frontend.
- Enforces crash policy: FC and Sim are run-lifecycle peers (either crashing ends the run); GS is observational (its crash leaves FC and Sim running).
- Provides Restart and Shutdown commands; no automatic retry.

**Status:** Deferred until M3 is stable.

---

## Status summary

| Milestone | Task | Artifact | Status |
|---|---|---|---|---|
| M1.1 | Proto feature gating | `spec.md §9` + `proto` features | Done |
| M1.2 | FC library cleanup: `impl_software` → `impl_sim` rename | `spec.md §10` + `flight-computer` features | Done |
| M1.3 | Task lifecycle separation: `run_flight_computer` + cooperative storage | `spec.md §6.6` + `flight-computer` tasks | Done |
| M2.1 | `flight-computer-host` binary | `flight-computer-host/src/main.rs` + `dispatch.rs` + `config.rs` | Done |
| M2.2 | Simulator binary (MVP) | `code/simulator/` — physics + FC client + scripted + minimal TUI | Ready |
| M2.3 | Simulator 3D physics (spatial crate + kinematic attitude) | `code/spatial/` crate + `code/simulator/` physics | Not started |
| M2.3b | Full 6-DOF rotational dynamics | — | Deferred |
| M2.4 | Simulator config, lifecycle & interactive TUI | — | Deferred |
| M3.1 | GS backend: REST API + storage (FC-facing) | Spec | Blocked (M2) |
| M3.2 | GS frontend TUI | Spec | Blocked (M3.1 REST contract) |
| M3.3 | Simulator-GS integration (state, config, lifecycle) | Spec | Blocked (M2.4 + M3.1) |
| M4 | `xtask run-host` orchestration | — | Deferred (after M3) |

---

## Implementation order

```
[M1.1] Proto feature gating (spec.md §9 + proto features)
          ↓
[M1.2] FC library cleanup (spec.md §10 + flight-computer features)
          ↓
[M1.3] Task lifecycle separation
          ↓
[M2.1] flight-computer-host binary (Done)
          ↓
[M2.2] Simulator binary MVP ── physics + FC client + scripted + minimal TUI
          ↓
          ├── [M2.3] Spatial crate + 3D physics ── kinematic attitude
          ├── [M2.3b] Full 6-DOF rotational dynamics (deferred)
          ├── [M2.4] Config, lifecycle & interactive TUI (deferred)
          ↓
[M3.1] GS backend (FC-facing)
          ↓
[M3.2] GS frontend TUI
          ↓
[M3.3] Simulator-GS integration (state → config → lifecycle)
          ↓
[M4] xtask run-host orchestration
```

---

## Progress

<!-- Checkboxes track completion. Update as work progresses. -->

### Milestone 1 — Wire vocabulary and FC library
- [X] M1.1 — Proto feature gating
- [X] M1.2 — FC library cleanup: `impl_software` → `impl_sim` rename
- [X] M1.3 — Task lifecycle separation: `run_flight_computer` + cooperative storage

**M1 progress:** 3 / 3 (100%)

### Milestone 2 — Independent binaries (FC-host + Simulator)
- [X] M2.1 — `flight-computer-host` binary
- [X] M2.2 — Simulator binary (MVP)
- [ ] M2.3 — Simulator 3D physics (spatial crate + kinematic attitude)
- [ ] M2.3b — Full 6-DOF rotational dynamics (deferred)
- [ ] M2.4 — Simulator config, lifecycle & interactive TUI

**M2 progress:** 2 / 5 (40%)

### Milestone 3 — Ground station (GS backend + GS frontend + sim integration)
- [ ] M3.1 — GS backend: REST API + storage (FC-facing)
- [ ] M3.2 — GS frontend TUI
- [ ] M3.3 — Simulator-GS integration (state → config → lifecycle)

**M3 progress:** 0 / 3 (0%)

### Milestone 4 — Orchestration (`xtask run-host`)
- [ ] M4 — `xtask run-host` orchestration

**M4 progress:** 0 / 1 (0%)

---

**Overall progress:** 4 / 12 tasks (33%)

---

## ADR-002 — Async timeout strategy for infinite loop error paths

**Goal:** Protect all infinite loop `.await` calls against indefinite hangs using `embassy_time::with_timeout`, with per-domain timeout constants and proper cancellation-safety handling. See [ADR-002](ADR/ADR-002-async-timeout-strategy.md) for rationale.

### Implementation tasks

| # | File | Change | Status |
|---|---|---|---|
| 1 | `code/flight-computer/src/config.rs` | Add `WRITE_TIMEOUT`, `FLUSH_TIMEOUT` to `StorageConfig`; `PUBLISH_TIMEOUT` to `GroundStationConfig` | Done |
| 2 | `code/flight-computer/src/tasks/sensor.rs` | Wrap `parse_new_data()` in `with_timeout(1.5 × tick_interval, ...)` inside `join()` | Done |
| 3 | `code/flight-computer/src/tasks/storage.rs` | Wrap `append_record()` and `flush()` in `with_timeout(...)`; log & continue on timeout | Done |
| 4 | `code/flight-computer/src/tasks/groundstation.rs` | Wrap both `send_to_ground_station()` calls in `with_timeout(...)`; log & continue on timeout | Done |
| 5 | `code/flight-computer/src/core/state_machine/detectors/apogee_detector.rs` | Wrap `wait_new_data_and_update_buffers()` in `with_timeout(half_tick, ...)`; skip on timeout | Done |
| 6 | `code/flight-computer/src/core/state_machine/detectors/touchdown_detector.rs` | Same pattern as apogee | Done |
| 7 | `code/flight-computer/src/core/state_machine/states/armed.rs` | Replace `deploy() + Timer::after(1s)` with `with_timeout(1s, deploy())`; add `verify_deployment()` step | Done |
| 8 | `code/flight-computer/src/interfaces/deployment_system.rs` | Add `verify_deployment()` as required method | Done |
| 9 | `code/flight-computer/src/interfaces/impls/simulation/deployment_system.rs` | Change error type from `Infallible`; return `Err` on publish failure; implement `verify_deployment()` | Done |
| 10 | `code/flight-computer/src/interfaces/impls/embedded/deployment_switch.rs` | Implement `verify_deployment()` as `unimplemented!()` with doc comment | Done |

**Excluded:**
- `tasks/postcard.rs` — `server.run()` only returns on error; no change.
- `states/pre_armed.rs` — `select` + `Ticker` is the correct pattern for polling; no change.

**Progress:** 10 / 10 tasks (100%)

---
