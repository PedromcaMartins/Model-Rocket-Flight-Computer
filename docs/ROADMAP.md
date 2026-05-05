# Host-Stack Roadmap

Tracks the work to split the monolithic HOST binary into four independent processes (FC, simulator, GS backend, GS frontend) connected by postcard-rpc, as decided in [ADR-001](ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md).

Each task below requires a written artifact before implementation begins: an **ADR** for structural/design decisions (tasks 1–3), a **Spec** for full subsystem work (tasks 4–7).

---

## Status

| # | Task | Artifact | Status |
|---|------|----------|--------|
| 1 | Proto feature gating | ADR-002 | Not started |
| 2 | Implement feature gating in code | ADR-003 | Not started |
| 3 | Update `flight-computer` crate | ADR-003 | Not started |
| 4 | Create `flight-computer-host` binary | Spec | Blocked (connection diagram) |
| 5 | Simulator: postcard-rpc server binary | Spec | Blocked (connection diagram) |
| 6 | GS-backend: REST API + storage | Spec | Blocked (connection diagram) |
| 7 | GS-frontend: ratatui TUI | Spec | Blocked (connection diagram) |

> **Blocker for 4–7:** the FC ↔ Simulator ↔ GS connection diagram (to be provided). Open questions: how GS-backend connects to FC-host in HOST mode, and whether GS talks to the simulator directly or only through FC.

---

## Task 1 — Proto feature gating (ADR-002)

**Problem:** all `Sim*` topics are always compiled into `proto`, including on embedded HW targets where they waste flash.

**Proposed feature structure:**

```toml
[features]
default = ["log", "embassy-time", "timestamp-into-duration"]   # HW-safe
simulator-endpoints = []          # gates all Sim* topics/endpoints
ipc-adapter = ["dep:interprocess", "dep:tokio"]   # InterprocessWireTx/Rx
host = ["simulator-endpoints", "ipc-adapter"]     # HOST binary
pil  = ["simulator-endpoints"]                    # PIL firmware
```

**What stays in default (HW):**
- `PingEndpoint`, `GlobalTickHzEndpoint`, `RecordTopic`

**What moves behind `simulator-endpoints`:**
- `SimAltimeterTopic`, `SimGpsTopic`, `SimImuTopic`, `SimArmTopic`
- `SimDeploymentTopic`, all `Sim*LedTopic`s (×8)

**What goes in `ipc-adapter`:**
- `InterprocessWireTx` / `InterprocessWireRx` — thin wrappers over `tokio::io::split` halves of `interprocess::local_socket::tokio::Stream`, implementing postcard-rpc's `WireTx` / `WireRx` traits.

**Files:**
- `code/proto/Cargo.toml`
- `code/proto/src/lib.rs` — add `#[cfg(feature = "simulator-endpoints")]` guards
- `code/proto/src/ipc/mod.rs` — new file, gated behind `ipc-adapter`

---

## Task 2 & 3 — flight-computer feature cleanup + impl_host IPC peripherals (ADR-003)

**Feature changes:**

| Old | New | Notes |
|---|---|---|
| `impl_embedded` | `impl_embedded` | unchanged |
| `impl_software` | `impl_software` | sim-fed peripheral impls; PIL reuses these |
| `impl_host` | `impl_host` | **replaces** in-process Embassy wiring with IPC client peripherals |

Default features change from `["log", "impl_embedded", "impl_software", "impl_host", "std"]` to `["log", "std"]` — consumers opt in.

**What changes in `impl_host`:**
- `src/interfaces/impls/host/` — replace stub with real postcard-rpc client peripheral impls (`SensorClient`, `ArmingClient`, `DeploymentClient`, `LedClient`) that call through `proto`'s IPC adapter.
- `src/tasks/simulation.rs` — rename `start_sil_flight_computer` → `start_host_flight_computer`; signature takes a socket path (or connected `HostClient`), not an in-process server.

**Scope:** only `code/flight-computer/`. Other crates stay broken until tasks 5–6.

**Verification:**
```
cargo check -p flight-computer --no-default-features --features impl_embedded
cargo check -p flight-computer --no-default-features --features impl_host
```

---

## Task 4 — `flight-computer-host` binary (Spec)

**Binary placement:** new workspace crate `code/flight-computer-host/` (native toolchain; stays inside workspace unlike the future `cross-*` crates which need different toolchains and live outside).

**Responsibilities:**
- Connect to simulator's postcard-rpc server (socket path from CLI arg or config).
- Start FC task set: sensor tasks, FSM, storage, GS relay.
- Tracing via `tracing` + `tracing-subscriber` (console + file sink).
- Filesystem via existing `HostFileSystem`.
- GS connection: **pending diagram**.

---

## Task 5 — Simulator binary (Spec)

**Reuse from current `code/simulator/`:**
- Physics engine (`engine.rs`, `physics/state.rs`) — kinematic parabolic; keep as-is.
- Config (`config.rs`) — keep.
- `PhysicsState` → sensor data conversions — keep.
- `runtime/commands.rs` (`SimulatorCommand`, `FlightComputerCommand`) — these become postcard-rpc message types.

**What needs building:**
- `[[bin]]` entry point in `code/simulator/Cargo.toml` (or new `code/simulator-bin/`).
- postcard-rpc **server** using `proto/host` (`InterprocessWireTx/Rx` over `fc-sim.sock`). Simulator listens; FC connects.
- Handlers for incoming FC commands (deployment events, LED status) as postcard-rpc Endpoint handlers.
- Physics tick loop that publishes sensor data as postcard-rpc Topics.
- GS connection: **pending diagram**.

**Physics scope for initial version:** parabolic 1D trajectory (motor burn → coast → apogee → descent). Enough to verify FC FSM responds correctly. Fault engine and wind deferred.

---

## Task 6 — GS-backend: REST API + storage (Spec)

**What it becomes:**
- Subscribes to FC telemetry (`RecordTopic`) over its link to FC-host.
- Stores records to disk (append-only, one file per session).
- Exposes REST endpoints consumed by the TUI frontend.
- Does **not** talk to the simulator directly — all sim traffic stays on the FC ↔ Sim socket.

**Framework:** keep Rocket 0.5; fix the broken lib.rs imports and missing REST handlers.

**GS ↔ FC connection:** **pending diagram**.

---

## Task 7 — GS-frontend TUI (Spec)

**Framework:** `ratatui`.

**Minimum viable screens:**
- Live telemetry: altimeter, GPS, IMU, flight state.
- Log tail.
- Manual controls: arm trigger, ignition, deploy recovery.
- Simulation status: running / paused / ended.

Full spec deferred until the GS ↔ backend REST contract is settled (task 6).

---

## Implementation order

```
[1] ADR-002: proto feature gating
        ↓
[2+3] ADR-003: flight-computer cleanup + impl_host IPC peripherals
        ↓
    [Connection diagram received]
        ↓
[4] flight-computer-host binary    [5] simulator binary
        ↓                                  ↓
        └──────── both running ────────────┘
                    ↓
            [6] GS-backend
                    ↓
            [7] GS-frontend TUI
```
