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
  │   FC tasks ←─── in-process signals ───→ simulator   │
  │                                                      │
  │   GS (partial, broken) ←─── direct fn calls         │
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
             │     fc-sim.sock                 │
             │     postcard-rpc                │
             └──── sensors / arming / ─────────┘
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

**Status:** Blocked — requires M1 complete and connection diagram confirmed.

### M2.2 — Simulator binary

Standalone binary in (or alongside) `code/simulator/`.

Architectural role:
- Serves postcard-rpc over `fc-sim.sock`; FC connects to it.
- Physics tick loop publishes sensor Topics; FC deployment/LED calls arrive as Endpoint requests.
- Two-phase lifecycle (Setup → Runtime) controlled by GS over `sim-gs.sock`.
- Owns its own ratatui TUI for independent operator access (physics state, force events, LED indicators, sim log).
- Config loaded from CLI path; hash computed at setup, sent to GS on connect.

Physics scope for initial version: parabolic 1D trajectory (motor burn → coast → apogee → descent). Sufficient to exercise the full FC FSM.

**Status:** Blocked — requires M1 complete and connection diagram confirmed.

---

## Milestone 3 — Ground station (GS backend + GS frontend)

**Goal:** The full four-process HOST topology from spec.md is running. An operator can start a scenario, observe live telemetry, issue commands, and see crash/disconnect state correctly reflected in the UI.

### M3.1 — GS backend: REST API + storage

Crate `code/ground-station-backend/`.

Architectural role:
- Connects to FC on `fc-gs.sock` (telemetry subscriber, command issuer).
- Connects to Sim on `sim-gs.sock` (lifecycle control: Start / Restart / Shutdown; config-hash handshake; manual trigger relay).
- Source of truth for scenario config files; supplies config path to simulator at launch.
- Stores FC telemetry records to disk (append-only, one file per session).
- Exposes REST/JSON API consumed exclusively by the frontend — GS frontend never speaks postcard-rpc.
- Independent of the simulator for its core function: if `sim-gs.sock` is absent, GS continues operating on FC telemetry alone.

**Status:** Blocked — requires M2 running and `sim-gs.sock` contract settled.

### M3.2 — GS frontend TUI

New ratatui binary.

Architectural role:
- Pure REST client of GS backend; no direct postcard-rpc.
- Minimum viable screens: live telemetry (altimeter, GPS, IMU, flight state), log tail, manual controls (arm, ignition, deploy), simulation status and phase indicator.
- Disconnect UX: affected panel turns red within one UI refresh; last-known state stays visible, dimmed, with stale-indicator badge; Restart / Shutdown buttons available; no automatic reconnect retry.

**Status:** Blocked — requires M3.1 REST contract settled.

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
|---|---|---|---|
| M1.1 | Proto feature gating | `spec.md §9` + `proto` features | Not started |
| M1.2 | FC library cleanup: `impl_software` → `impl_sim` rename + `start_*` builder | `spec.md §10` + `flight-computer` features | Done |
| M1.3 | Task lifecycle separation: `run_flight_computer` + cooperative storage | `spec.md §6.6` + `flight-computer` tasks | Pending |
| M2.1 | `flight-computer-host` binary | Spec | Blocked (M1 + connection diagram) |
| M2.2 | Simulator binary | Spec | Blocked (M1 + connection diagram) |
| M3.1 | GS backend: REST API + storage | Spec | Blocked (M2) |
| M3.2 | GS frontend TUI | Spec | Blocked (M3.1 REST contract) |
| M4 | `xtask run-host` orchestration | — | Deferred (after M3) |

> **Blocker for M2:** the FC ↔ Simulator ↔ GS connection diagram must be confirmed before M2 specs are written. Open questions resolved by spec.md: GS-backend connects to FC-host via `fc-gs.sock` and to the simulator via `sim-gs.sock`; GS does **not** talk to the simulator's peripheral surface (`fc-sim.sock`) directly.

---

## Implementation order

```
[M1.1] Proto feature gating (spec.md §9 + proto features)
          ↓
[M1.2] FC library cleanup (spec.md §10 + flight-computer features)
          ↓
   [Connection diagram confirmed]
          ↓
[M2.1] flight-computer-host      [M2.2] simulator binary
          ↓                               ↓
          └──────── both running ─────────┘
                        ↓
              [M3.1] GS backend
                        ↓
              [M3.2] GS frontend TUI
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
- [ ] M2.1 — `flight-computer-host` binary
- [ ] M2.2 — Simulator binary

**M2 progress:** 0 / 2 (0%)

### Milestone 3 — Ground station (GS backend + GS frontend)
- [ ] M3.1 — GS backend: REST API + storage
- [ ] M3.2 — GS frontend TUI

**M3 progress:** 0 / 2 (0%)

### Milestone 4 — Orchestration (`xtask run-host`)
- [ ] M4 — `xtask run-host` orchestration

**M4 progress:** 0 / 1 (0%)

---

**Overall progress:** 2 / 8 tasks (25%)
