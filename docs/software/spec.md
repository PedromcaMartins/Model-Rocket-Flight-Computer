# Software architecture — Consolidated Spec

- **Status:** draft
- **Date:** 2026-05-05

The single source-of-truth spec for the rocket's software stack. Covers *what* every component does, *why* it is shaped the way it is, *what crosses every boundary*, and the crate-level layout that realises the architecture.

---

## 1. Goals & non-goals

### Goals

- **One FC library, three deployment targets.** Same Rust code on the production MCU (HW), on the production MCU with simulator-fed sensors over USB (PIL), on a host machine with a separate simulator process (HOST). No mode-specific branches in the FC core.
- **Peripheral-agnostic FC.** All hardware interaction is via traits owned by the FC library; implementations are supplied at link time by the binary that consumes the library.
- **Single wire vocabulary.** All telemetry, commands, sim peripheral data, and sim control share the postcard-rpc Topic / Endpoint definitions in `proto/`. The transport medium swaps; the message vocabulary does not.
- **Testable in software.** All testing is in software — HOST for full-stack scenario testing, PIL for performance / firmware testing on the prod board. No hardware mocks, fixtures, or stimulator boards exist.
- **Auditable scenarios.** Sim scenario config is human-readable, validated at load, hash-locked at runtime — operator and sim cannot silently disagree about what is being simulated.
- **Deterministic simulator.** Sim is deterministic for a given config + scripted scenario; re-running with the same inputs is the defined reproduction path.
- **Observability built in.** Cross-process trace correlation between FC, simulator, and GS from day one (host); embedded staged.
- **No silent failures.** Disconnects and crashes are always visible to the operator within one UI refresh.

### Non-goals

- **Not a framework.** This is the flight software for *this* rocket; generalising to other rockets is out of scope.
- **No real-time fidelity in software modes.** Host execution is untimed; timing-sensitive bugs are reproduced in PIL.
- **No hot config reload in the simulator.** Config is frozen at the setup→runtime transition.
- **Sim is not a general physics engine.** The engine models this rocket's parabolic flight envelope; generalising it is out of scope.
- **No simulation of what the FC cannot observe.** If no peripheral / FSM input depends on it, it is not simulated.
- **No hybrid Sim/HW mode.** A given FC binary uses *either* real-driver peripherals *or* simulator-fed peripherals — never a mix.
- **No automatic process recovery.** xtask and the operator restart things; surviving processes do not.
- **No cross-machine availability.** HOST is single-developer; HA is out of scope.
- **No replay.** Recording and replaying simulation runs is out of scope. The simulator is deterministic for a given config + scripted scenario; re-running with the same inputs is the defined reproduction path. Replay infrastructure is a future concern.

---

## 2. System at a glance

### HOST topology

```
host machine (HOST mode)
========================

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
          │ REST + storage + config               │ ◄─── ( scenario
          │ source of truth                       │       config file )
          └──────┬──────────────────────────┬─────┘             │
                 │                          │                   │
        fc-gs.sock                   sim-gs.sock                │
        postcard-rpc                 postcard-rpc               │ CLI path
        telemetry /                  lifecycle / triggers /     │
        commands                     status / hash              │
                 │                          │                   │
                 ▼                          ▼                   ▼
    ┌──────────────────────────┐    ┌─────────────────────────────────┐
    │ flight-computer-host     │    │ simulator                       │
    │ (FC library + impl_host) │    │ (physics + events)              │
    └────────────┬─────────────┘    └────────────┬────────────────────┘
                 ▲                               │
                 │      fc-sim.sock              │
                 │      postcard-rpc             │
                 └──── sensors / arming / ───────┘
                       deploy / LED
```

In HW the FC binary lives on the production MCU and talks to GS over USB / radio with the same postcard-rpc vocabulary; the simulator is absent. In PIL the FC binary is the production firmware on the production MCU, the simulator stays on host, and `fc-sim.sock` is replaced by the same multiplexed USB postcard link the GS uses.

### Per-mode topologies

```
HW — production flight
══════════════════════
   ┌─────────────────────────┐  USB / radio  ┌────────────────┐
   │ FC firmware             │ ────────────► │ GS backend     │
   │ cross-* binary          │               └────────────────┘
   │ impl_embedded           │
   └─────────────┬───────────┘
                 │
   ┌─────────────┴───────────────┐
   │ real sensors / actuators    │
   └─────────────────────────────┘


HOST — full-stack scenario testing
══════════════════════════════════
   ┌──────────────────────┐                    ┌──────────────────────┐
   │ simulator (host)     │ ◄── interprocess ─►│ flight-computer-host │
   │                      │     socket         │ impl_host            │
   └──────────┬───────────┘                    └──────────┬───────────┘
              │                                           │
              │  interprocess         interprocess        │
              │  socket               socket              │
              │                                           │
              ▼                                           ▼
   ┌────────────────────────────────────────────────────────────┐
   │                       GS backend                           │
   └────────────────────────────────────────────────────────────┘


PIL — perf testing on prod board
════════════════════════════════
   ┌──────────────────────┐                ┌──────────────────────┐
   │ simulator (host)     │ ◄── USB ─────► │ FC firmware          │
   │                      │                │ cross-* binary       │
   │                      │                │ impl_software        │
   └──────────┬───────────┘                └──────────┬───────────┘
              │                                       │
       interprocess                       same USB wire,
       socket                             multiplexed
              │                                       │
              ▼                                       ▼
   ┌────────────────────────────────────────────────────────────┐
   │                    GS backend (host)                       │
   └────────────────────────────────────────────────────────────┘
```

| Mode | Where FC runs | Sensor source | Filesystem | Primary purpose |
|---|---|---|---|---|
| **HW** | Production MCU | Real drivers (`impl_embedded`) | SD / flash | Production flight |
| **HOST** (SIL/SITL) | Host process (`flight-computer-host`) | Simulator process via `fc-sim.sock` (`impl_host`) | Host FS | Full-stack scenario testing |
| **PIL** | Production MCU | Simulator on host via USB (`impl_software`) | SD / flash | Performance / firmware testing on the prod board |

---

## 3. Components

| Component | Crate | Std/no_std | Role |
|---|---|---|---|
| **FC library** | [`flight-computer`](../../code/flight-computer/) | `no_std` core, `std` test utils | Hardware-agnostic flight software core. Sensor traits, FSM, deployment logic, telemetry tasks. Linked by every FC binary. |
| **FC binary (host)** | `flight-computer-host` (planned) | `std` | Host-side FC binary. Links FC library with `impl_host` peripherals (postcard-rpc clients over interprocess sockets). |
| **FC binary (HW / PIL)** | `cross-esp32-s3`, `cross-nucleo-f413zh` | `no_std` | Embedded FC binaries. HW and PIL firmware are sibling binaries inside each `cross-*` crate (HW links `impl_embedded`, PIL links `impl_software`). Live outside the workspace because they need different toolchains. |
| **Simulator** | [`simulator`](../../code/simulator/) | `std` | Host-side process. Physics engine, scripted scenarios, force/trigger event model. Drives the FC's peripheral surface in HOST and PIL. Owns its own structured log and exposes a ratatui TUI for real-time inspection and configuration. |
| **Simulator TUI** | (planned, ratatui TUI, part of `simulator` binary) | `std` | Read-write operator interface embedded in the simulator process. Displays live physics state, active force events, LED indicator state, and sim-side log. Accepts manual trigger commands and config-phase controls independently of GS. |
| **GS backend** | [`ground-station-backend`](../../code/ground-station-backend/) | `std` | Host-side process. Telemetry consumer, command issuer, REST/JSON server for the frontend. Source of truth for scenario config files. Operates independently of whether the simulator is present. |
| **GS frontend** | (planned, ratatui TUI) | `std` | Operator UI. REST client of the backend; never speaks postcard-rpc. |
| **`proto`** | [`proto`](../../code/proto/) | `no_std` | Wire-format contract: postcard-rpc Topics and Endpoints, message types, `InterprocessWireTx`/`Rx` adapter. Shared by every component above. |
| **`xtask`** | [`xtask`](../../code/xtask/) | `std` | Project task runner. In HOST, also the orchestrator that spawns and supervises the host processes. |

---

## 4. Cross-component invariants

These properties hold across every component. Each is a deliberate design choice with consequences in many places.

| Invariant | Why | Where enforced |
|---|---|---|
| **One FC library, three deployment targets** | Avoid drift between flight code and test code | No mode-specific branches in `flight-computer/src/` |
| **Peripheral-agnostic via traits** | Swap real ↔ sim without touching FC core | FC library imports no driver, simulator type, or transport crate |
| **Runtime-agnostic async** | Embassy on HW/PIL, Tokio on host | FC library uses `async fn` only; never spawns or selects an executor |
| **Architecture-agnostic core** | Compiles for RISC-V, ARM Cortex-M, x86/x64 | `no_std`-clean for embedded; `std` only via opt-in feature |
| **Single wire vocabulary** | One GS UI works in HW, PIL, HOST | All Topics/Endpoints live in `proto/`; only transport adapter differs per medium |
| **Event-driven FC FSM** | Determinism + decoupled telemetry cadence | FSM has no loop rate; transitions execute on events only |
| **Production targets have no shutdown path** | A reachable shutdown on prod firmware is a safety regression | Sim/orchestration shutdown logic gated behind `impl_host` / `impl_software` features; CI verifies |
| **Sim/HW strictly one-or-the-other** | Hybrid (e.g. real IMU + sim GPS) adds complexity with no current need | Mutually-exclusive feature flags at link time |
| **GS is simulator-independent** | GS operates on FC telemetry and commands alone; removing the simulator leaves GS fully functional. No GS code path requires the simulator to be present or to have produced data. | GS backend imports no simulator type; `sim-gs.sock` connection is optional from GS's perspective — its absence is a degraded but valid state |
| **Simulator is GS-independent for physics** | The simulator's physics loop, internal log, and TUI operate without a GS connection. GS connectivity is required only for lifecycle control and config-hash handshake; the absence of GS does not halt or corrupt the sim loop. | Sim TUI provides independent operator access; `sim-gs.sock` connection loss follows the crash policy in §10 |

---

## 5. The boundaries

Three component pairs, each with its own contract.

### What flows where

```
  ┌────────────────────────┐                ┌────────────────────────┐
  │ simulator              │                │ FC binary              │
  │                        │  sensor data   │                        │
  │ ┌────────────────────┐ │  Topics        │ ┌────────────────────┐ │
  │ │ physics engine     │ │ (altimeter,    │ │ sensor tasks       │ │
  │ └─────┬──────────┬───┘ │  GPS, IMU,arm) │ └─────────┬──────────┘ │
  │       │          │     │  ─────────────►│           ▼            │
  │       │          │     │                │ ┌────────────────────┐ │
  │ ┌─────┴─────┐ ┌──┴───┐ │  deployment /  │ │ FSM                │ │
  │ │force      │ │trigger│ │  LED Endpoints │ └─────────┬──────────┘ │
  │ │events     │ │events │ │ ◄───────────── │           ▼            │
  │ │(thrust,   │ │(ignite│ │                │ ┌────────────────────┐ │
  │ │ drag,...) │ │ arm,  │ │                │ │ telemetry / record │ │
  │ │           │ │deploy)│ │                │ └─────────┬──────────┘ │
  │ └───────────┘ └───────┘ │                │           │            │
  └───────────┬────────────┘                 └───────────┼────────────┘
              ▲                                          │
              │                                          │ telemetry Topics
   lifecycle (Start/Restart/Shutdown)                    │ (records, events,
   manual triggers (Ignite/Arm/Deploy)                   │  errors, stats)
   config-hash handshake                                 ▼
              │              ┌──────────────────────────────────────┐
              │              │ GS backend + frontend                │
              └────────────► │             ( operator )             │
                             │                                      │
              sim phase /    │                                      │
              physics status │ command Endpoints                    │
              Topics         │ (ping, runtime config)               │
              ◄───────────── │ ─────────────────────────────────────┤
                             └──────────────────────────────────────┘
```

### 5.1 FC ↔ Simulator (peripheral surface)

The FC's peripheral traits define this boundary:

| Trait | Direction | Role |
|---|---|---|
| [`Sensor`](../../code/flight-computer/src/interfaces/sensor.rs) | sim → FC | Periodic sensor data. One impl per device (altimeter, GPS, IMU); each has a `TICK_INTERVAL` and an `async parse_new_data`. |
| [`ArmingSystem`](../../code/flight-computer/src/interfaces/arming_system.rs) | user → FC (via sim) | The FC waits on `wait_arm`; in HOST/PIL the simulator (or operator / scripted scenario) signals it. |
| [`DeploymentSystem`](../../code/flight-computer/src/interfaces/deployment_system.rs) | FC → sim | The FC calls `deploy` to fire the parachute / recovery actuator. The simulator observes it and spawns the parachute drag `ForceEvent`. |
| [`Led`](../../code/flight-computer/src/interfaces/led.rs) | FC → sim | Status indicators (`on` / `off` / `toggle`). The simulator surfaces LED state on its TUI. If the information encoded in an LED is operationally significant, it is also transmitted to GS as a distilled status value in the FC telemetry stream — not as raw LED on/off calls. GS never reads LED state directly from `fc-sim.sock`. |

**Deployment verification.** The `DeploymentSystem::deploy` call travels directly from FC to Sim over `fc-sim.sock` via the peripheral interface. GS does not observe this call directly. Verification is provided by the FC's `telemetry_task`, which emits a deployment event Topic on `fc-gs.sock` at the moment the FC calls `deploy`. This gives GS an independent, FC-authored confirmation without requiring GS to participate in the peripheral boundary.

**FSM transition visibility.** All FC FSM state transitions are emitted as telemetry Topics on `fc-gs.sock` by the `telemetry_task`. GS observes the full FSM history through this channel. No FSM information is transmitted over `fc-sim.sock`; the simulator is not FSM-aware.

**Command origins** (for the simulator's bookkeeping): **ignition** is user-driven (operator from GS, or scripted scenario); **deployment** is FC-driven. Both reach the simulator through its peripheral interface but originate at different ends.

`FileSystem` is the only peripheral trait with real I/O in every mode (host FS in HOST, SD/flash in HW and PIL); it is **not** part of the FC ↔ Sim boundary.

Carried over `fc-sim.sock` (HOST) or USB (PIL).

**Sensor publish rate and buffering.** The simulator publishes sensor Topics on a tick cadence (10 Hz is the current target as an estimate; actual rate is a tuning parameter, not an architectural commitment). Raw sensor data is buffered between sim ticks and FC reads using postcard-rpc's buffering primitives so intermediate samples are not silently dropped — *why:* a naive last-value-wins approach drops detail that cannot be reconstructed for post-flight analysis.

**Sim's event taxonomy is invisible across this boundary.** Force events stay inside the simulator and influence sensor readings indirectly; trigger events surface to the FC only through the existing peripheral traits (e.g. an arm trigger arrives as a `Sensor::parse_new_data` resolution on the arming Topic).

### 5.2 FC ↔ Ground station (telemetry & commands)

| Direction | Kind | Examples |
|---|---|---|
| FC → GS | Topics (pub-sub) | `RecordTopic`, FSM state transitions, deployment event, arming event, LED-derived status values, errors, periodic stats |
| GS → FC | Endpoints (service) | Ping, runtime config tweaks, manual triggers (HOST/PIL only) |

Carried over `fc-gs.sock` (HOST) or USB / radio (HW, PIL). Wire types live in [`../../code/proto/`](../../code/proto/).

### 5.3 Simulator ↔ Ground station (sim control & status)

| Direction | Kind | Examples |
|---|---|---|
| GS → Sim | Endpoints | Lifecycle (`Start`, `Restart`, `Shutdown`); manual triggers (`Activate Arm`, `Motor Ignition`, `Deploy`) |
| Sim → GS | Topics | Sim phase (`Setup` / `Running`), physics status (sim time, vehicle state summary, active force-event list) |
| Sim → GS | Handshake | Config hash sent on connect |

Carried over `sim-gs.sock`. **No FC traffic on this link** — sim-control stays off the flight code path. *Why:* decouples sim release cadence from FC release cadence; operator-only concerns never burden the flight code.

**Scope constraint.** This link carries only sim lifecycle and physics status. It does not carry sensor data, peripheral surface traffic, LED state, FSM transitions, or telemetry Records. GS does not need — and must not be designed to depend on — simulator-internal data to perform its role. *Why:* GS must remain fully functional when the simulator is absent (HW mode, or HOST mode with sim not yet started). Any GS logic that requires sim data to function is an architectural coupling that this link explicitly prohibits.

**Manual triggers on this link vs the TUI.** GS can issue manual triggers (`Activate Arm`, `Motor Ignition`, `Deploy`) over this link, and these are the canonical path for operator-commanded scenario events. The Sim TUI provides the same trigger capability independently, as a local operator interface. Both paths converge on the same `TriggerEvent` bus inside the simulator (§7.5).

### 5.4 GS backend ↔ GS frontend

REST / JSON over HTTP. The frontend never speaks postcard-rpc. Isolates UI iteration from the wire vocabulary.

---

## 6. FC library — internal structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│ flight-computer (library, no_std core)                                  │
│                                                                         │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ Tasks                                                           │   │
│   │ sensor_task, fsm_task, storage_task, telemetry_task             │   │
│   └────────────┬────────────────────┬───────────────────────┬───────┘   │
│                │                    │                       │           │
│                ▼                    ▼                       │           │
│   ┌──────────────────────┐  ┌─────────────────────────────┐ │           │
│   │ Flight FSM           │◄─│ Core logic                  │ │           │
│   │ event-driven,        │  │ apogee detector, landing    │ │           │
│   │ no loop rate         │  │ detector, deployment        │ │           │
│   └──────────────────────┘  └────────────────┬────────────┘ │           │
│                                              │              │           │
│                                              ▼              ▼           │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ Peripheral traits                                               │   │
│   │ Sensor · ArmingSystem · DeploymentSystem · Led · FileSystem     │   │
│   └─────────────────────────────────────────────────────────────────┘   │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │ implements
                ┌────────────────────┼────────────────────┐
                ▼                    ▼                    ▼
   ┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐
   │ impl_embedded       │ │ impl_software       │ │ impl_host           │
   │ real drivers via    │ │ postcard-rpc client │ │ postcard-rpc client │
   │ embedded-hal        │ │ over USB            │ │ over interprocess   │
   │                     │ │                     │ │ socket              │
   └─────────────────────┘ └──────────┬──────────┘ └──────────┬──────────┘
                                      │                       │
                                      └───────────┬───────────┘
                                                  ▼
                                  ┌─────────────────────────────┐
                                  │ proto/                      │
                                  │ Topics, Endpoints,          │
                                  │ wire types,                 │
                                  │ InterprocessWireTx/Rx       │
                                  └─────────────────────────────┘

  Peripheral impls — selected at link time by feature flag
  (impl_embedded, impl_software, impl_host are mutually exclusive)
```

### 6.1 Crate layout

```
code/flight-computer/src/
├── lib.rs
├── interfaces/
│   ├── sensor.rs              ← Sensor<Data, Error>: periodic data source
│   ├── arming_system.rs       ← ArmingSystem: waits for arm signal
│   ├── deployment_system.rs   ← DeploymentSystem: fires parachute actuator
│   ├── led.rs                 ← Led: on / off / toggle status indicator
│   ├── filesystem.rs          ← FileSystem: append-only record storage
│   └── impls/
│       ├── embedded/          ← impl_embedded
│       ├── software/          ← impl_software
│       └── host/              ← impl_host
├── tasks/                     ← sensor_task, fsm_task, storage_task, telemetry_task
└── core/                      ← FSM, apogee detector, landing detector, deployment logic
```

### 6.2 Feature flags

| Feature | What it enables |
|---|---|
| `impl_embedded` | Real hardware drivers (`embedded-hal`) — used in HW binaries |
| `impl_software` | Simulator-fed peripherals via postcard-rpc over USB — used in PIL firmware |
| `impl_host` | Simulator-fed peripherals via postcard-rpc over interprocess socket — used in the host binary |
| `std` | Standard library (required by `impl_software` and `impl_host`) |
| `log` | Logging via the `log` crate (default for host/test builds) |
| `defmt` | Logging via `defmt` (for embedded targets) |

The three `impl_*` flags are mutually exclusive at link time.

### 6.3 Async runtime dependency

The library calls `.await` but never spawns tasks or creates executors. Trait methods are runtime-neutral. The only temporal primitive used internally is `embassy_time::Timer` / `Ticker`, which is driven by a platform-supplied `embassy-time` driver.

The linking binary must provide:
- An `embassy-time` HAL driver (time source for `Ticker` and timeouts).
- A `critical-section` implementation (required by `embassy-sync` primitives used in inter-task channels).

On host these are provided by Tokio + the `embassy-time-driver-std` (or equivalent). On embedded they are provided by the target BSP crate.

### 6.4 FSM

- **Event-driven, no loop rate.** State transitions execute purely on incoming events (sensor sample crosses threshold, deployment ack arrives, etc.) and are deterministic.
- The 10 Hz-ish telemetry cadence is decoupled from FSM execution entirely — telemetry tasks read FSM state, not the other way around.
- *Why:* eliminates time-quantisation bugs; makes replays deterministic; no scheduling ambiguity.

### 6.5 Testing strategy

Trait-based abstraction is contract-tested on **both sides**:

| Test kind | What it tests | Where |
|---|---|---|
| **Trait consumer tests** | FC code that *uses* peripherals against expected behaviour, using mock implementations | `code/flight-computer/tests/` |
| **Trait implementer tests** | Each concrete implementation (real drivers, sim peripherals, host FS) against the same expected behaviour contract | Per-impl test modules |

Together these provide high confidence that real hardware and sim implementations are interchangeable without requiring cross-target integration tests for every change.

---

## 7. Simulator — lifecycle, config, events

### 7.1 Lifecycle — two phases

```
                  binary launch
                  (CLI path → config file)
                          │
                          ▼
   ┌─────────────────────────────────────────────────┐
   │  Setup                                          │ ◄──── GS issues Restart
   │  - validate config                              │       (re-read config file)
   │  - compute hash                                 │           ▲
   │  - listen on sim-gs.sock                        │           │
   │  - send hash on connect                         │           │
   └────────┬──────────────────────────────┬─────────┘           │
            │ GS issues Start              │ GS issues Shutdown  │
            ▼                              ▼                     │
   ┌─────────────────────────────────────┐ [exit]                │
   │  Runtime                            │                       │
   │  - tick physics                     │ ──────────────────────┘
   │  - fire scripted events             │
   │  - accept manual triggers           │ ── GS issues Shutdown ──► [exit]
   │  - publish sensor / status          │ ── peer (FC) crashes ───► [exit]
   └─────────────────────────────────────┘
```

**Setup phase:**
- Reads the config file passed via CLI argument.
- Validates every field (types, bounds, checksum — see §7.3).
- Computes a hash over the loaded config.
- Listens on `sim-gs.sock`. When GS connects, sim sends its hash for handshake.
- No physics, no event execution, no peripheral data published.

**Runtime phase** (entered after `Start` from GS):
- All config fields become **read-only**. There is no API to mutate them.
- Physics tick loop active.
- Scripted events fire as preconditions are met.
- Manual triggers from GS accepted.
- `Restart` and `Shutdown` from GS remain valid.

**Restart**: the binary tears down its runtime state and re-enters setup phase, re-reading the same config file. If the file changed, validation re-runs and the new hash is sent on the next handshake.

**Shutdown**: the binary exits. Re-running it is the operator's job (orchestrated via xtask).

Hot config swap is **not supported**. The shutdown → edit file → restart loop is the defined update path. *Why:* prevents partial-state runs where part of physics has run under config A and part under config B.

GS exposes operator controls reflecting these phases:

```
[Sim: SETUP]   → [Start] [Restart] [Shutdown]
[Sim: RUNNING] → [Restart] [Shutdown]
```

The current sim phase is broadcast to GS as a status Topic so the operator never has to guess.

### 7.2 Config — ownership

**Source of truth: GS.** The ground-station backend holds the canonical config file on disk. The simulator loads it from a path passed on the CLI; it does not read or accept config from any other source at runtime.

*Why GS owns it:* the operator's mental model of the scenario lives in GS. If the sim could mutate config behind GS's back, the operator's view would silently drift from reality. One place to edit, one place to look.

**Hash handshake.** On connect, sim sends its computed config hash on `sim-gs.sock`. GS compares against its own copy of the file. If hashes differ:
- GS rejects the simulator (does not enter the run UI).
- GS shows the mismatch reason and offers Restart / Shutdown.
- Operator updates the file and restarts the sim. On reconnect, the hash is re-verified.

There is no fallback to defaults on mismatch. Explicit operator action is required. *Why hash-only verification:* single-writer (GS) + checksum is sufficient; no atomic-write or merge story is needed.

**GS file watcher** is active **only at startup/load time**, not at runtime. It generates a hash on file change detection and debounces multiple filesystem events from a single save.

### 7.3 Config — validation

Validated at the start of setup phase, before the setup→runtime transition:

| Check | What it does |
|---|---|
| **Type parsing** | Fields parse to declared types |
| **Per-field bounds** | Physical sanity checks (no negative mass, drag in valid ranges, etc.) |
| **Checksum** | Verifies file was not corrupted in transit |

Any validation failure → simulator panics with a tracing-captured message and exits. *Why panic instead of falling back:* a sim run with the wrong constants invalidates the scenario but won't fail loudly mid-flight; the operator is far better served by a clear startup failure.

### 7.4 Config — what's in it

Only **scalar constants** that parameterise physics and event model:
- Vehicle mass (dry / wet)
- Drag coefficient (parachute open / closed)
- Motor thrust magnitude, burn duration
- Ignition delay, deployment activation altitude (for scripted scenarios)
- Tick rate

What does **not** live in config:
- **Magnitude functions themselves** — shapes (constant thrust, exponential drag falloff) are code; config tunes parameters only.
- **Vector directions** — deferred (see §11).
- **Magnitude time-series** — use scripted events instead.

*Why scalar-only:* keeps config human-readable, auditable, trivially serialisable, and trivially hashable.

### 7.5 Event model

Two distinct event categories. Both are concepts inside the simulator; they are not part of the FC contract.

#### Force events

```
ForceEvent {
    magnitude_fn,   // closure / function selected by event kind, parameterised by config scalars
    direction,      // deferred — see §11
    duration,       // sim-time duration
}
```

Examples: motor thrust, parachute drag.

**Per-tick handling:**
1. Iterate active force events.
2. Check expiry; mark expired for removal.
3. Compute magnitude via `magnitude_fn`.
4. **Sum vectorially** with all other active forces. Forces accumulate; they do not apply sequentially.
5. After the tick completes, remove expired events.

**Expiry policy:** force `duration` is checked at tick boundaries; no mid-tick pro-rating. *Why:* sim ticks are short relative to the time-scales of the forces involved; pro-rating adds non-trivial complexity for no measurable physical benefit. **Accepted limitation** — future contributors should not silently introduce inconsistent (mid-tick) expiry on a subset of force kinds.

#### Trigger events

```
TriggerEvent {
    signal_type,   // Ignition, Arm, Deployment, ...
}
```

Examples:
- Operator clicks "Activate Arm" → trigger on `sim-gs.sock` → arming Topic published toward FC on `fc-sim.sock`.
- Operator clicks "Motor Ignition" → trigger → spawns thrust `ForceEvent`.
- FC calls `DeploymentSystem::deploy` over `fc-sim.sock` → arrives in sim as a trigger → spawns parachute drag `ForceEvent`.

Trigger events drive sim state; they do not directly model physics. A trigger event typically *creates* one or more force events.

#### Scripted events

Config-loaded sequences that fire automatically as preconditions are met (time elapsed, altitude crossed, FSM state observed). Reduce to trigger events internally. Subject to the same hash + validation rules as the rest of config.

#### Event flow inside the simulator

```
   sources of triggers
   ───────────────────
   ┌────────────────────────┐
   │ GS manual              │──┐
   │ (Ignite/Arm/Deploy)    │  │
   └────────────────────────┘  │
   ┌────────────────────────┐  │      ┌───────────────────────┐
   │ scripted events        │──┼─────►│  TriggerEvent bus     │
   │ (time / altitude / FSM)│  │      └───────────┬───────────┘
   └────────────────────────┘  │                  │
   ┌────────────────────────┐  │       may spawn  │  may publish
   │ FC over fc-sim.sock    │──┘                  │
   │ (Deploy Endpoint)      │                     │
   └────────────────────────┘                     │
                                                  │
                ┌─────────────────────────────────┼──────────────┐
                ▼                                 │              ▼
   ┌──────────────────────────┐                   │   ┌─────────────────────────┐
   │ active ForceEvent set    │                   │   │ sim → FC Topics         │
   │ (thrust, drag, ...)      │                   │   │ (arm, sensor            │
   │                          │ ◄── expire on ──┐ │   │  adjustments)           │
   │                          │   tick boundary │ │   │                         │
   └────────────┬─────────────┘                 └─┘   └────────────┬────────────┘
                │ summed vectorially every tick                    ▲
                ▼                                                  │
   ┌──────────────────────────┐                                    │
   │ physics engine           │ ── per-tick state ─────────────────┘
   │ (state integrator)       │
   └──────────────────────────┘                              │
                                                             │ fc-sim.sock
                                                             ▼
                                                       ┌─────────────┐
                                                       │ FC binary   │
                                                       └─────────────┘
```

### 7.6 Production vs simulation build separation

Sim-related shutdown logic (xtask shutdown sequencing, sim-control commands, etc.) is **simulator-only** and **must not** ship in production builds:
- Gated behind a feature flag at compile time.
- A CI check confirms shutdown-related symbols cannot appear in production firmware artefacts.

*Why:* production embedded firmware has no shutdown path — only reset or watchdog recovery. A reachable shutdown code path on the production target is a safety regression.

### 7.7 Simulator TUI and internal log

The simulator embeds a **ratatui TUI** and maintains a **structured internal log**. Both are independent of the `sim-gs.sock` connection; they operate whether or not GS is connected.

**Purpose.** The TUI is the primary interface for real-time debugging and low-level sim inspection during development. It is deliberately separate from GS to reduce GS complexity and to allow the simulator to be operated in isolation (e.g. during simulator development, before the GS TUI is built, or when diagnosing a sim-side issue without operator UI noise).

**TUI capabilities (read):**
- Live physics state per tick: position, velocity, acceleration, active force-event list with magnitudes and remaining durations.
- Sim lifecycle phase (`Setup` / `Running`).
- LED indicator state: the current `on` / `off` / `toggle` state of each `Led` trait call from FC, displayed as labelled indicators.
- Sim-side structured log tail (last N entries).
- Config summary (loaded values, hash).

**TUI capabilities (write):**
- Manual triggers: `Activate Arm`, `Motor Ignition`, `Deploy` — same `TriggerEvent` bus as GS-issued triggers. Both paths are equivalent; there is no priority or ordering between them.
- Lifecycle controls: `Start`, `Restart`, `Shutdown` — same semantics as GS lifecycle Endpoints. When GS is also connected, either party can issue lifecycle commands; last-writer-wins.
- Config-phase controls only (no hot config mutation — §7.2 still applies).

**Internal log.** The simulator writes a structured log of its own state independently of the `tracing`/OTEL pipeline used for cross-process correlation:
- One entry per physics tick: sim time, summed force vector, vehicle state, list of active `ForceEvent`s.
- One entry per trigger event received and processed, with source (GS, TUI, scripted, FC-peripheral).
- One entry per lifecycle transition.
- Written to a local file (path configurable via CLI); also tailed live in the TUI.
- Format: newline-delimited JSON (human-readable, trivially parseable).

**Relationship to GS.** The TUI and internal log do not replace or duplicate GS. They are a sim-developer tool. GS provides the canonical operator interface (scenario config ownership, hash handshake, FC telemetry, high-level run status). The Sim TUI provides raw physics-level visibility that GS intentionally does not carry.

**LED state and GS.** LED state is displayed on the Sim TUI as the authoritative surface for raw indicator state. If the operational meaning of an LED is significant to the operator (e.g. a status LED that encodes FC arming readiness), the FC encodes that meaning as a named status value in its telemetry stream to GS — not as a raw LED call. GS displays the named value; the Sim TUI displays the raw LED state. There is no redundancy issue: they serve different audiences.

---

## 8. Host IPC

### 8.1 Three-socket triangle

```
                       ┌────────────────────────┐
                       │ flight-computer-host   │
                       └──┬──────────────────┬──┘
                          │                  │
                    fc-sim.sock          fc-gs.sock
                          │                  │
       ┌──────────────────┴──┐            ┌──┴───────────────────────┐
       │ simulator           ├─sim-gs.sock┤ ground-station-backend   │
       └─────────────────────┘            └──┬───────────────────────┘
                                             │
                                            REST
                                             │
                                          ┌──┴───────────────────────┐
                                          │ ground-station-frontend  │
                                          └──────────────────────────┘
```

| Socket | Server | Client | Traffic |
|---|---|---|---|
| `fc-sim.sock` | FC | Simulator | Sensor Topics (sim → FC); deployment / LED Endpoints (FC → sim) |
| `fc-gs.sock` | FC | GS backend | Telemetry Topics (FC → GS); command Endpoints (GS → FC) |
| `sim-gs.sock` | Simulator | GS backend | Sim status Topics (sim → GS); sim-control + manual-trigger Endpoints (GS → sim); config-hash handshake at connect |

Each link carries only the traffic that belongs to it. No link is a proxy for another. Ignition (GS operator → sim, on `sim-gs.sock`) and parachute deployment (FC → sim, on `fc-sim.sock`) flow on separate links without coupling.

### 8.2 Server / client invariants

- **The FC is the postcard-rpc server on every link it participates in.** It runs two server instances (one each on `fc-sim.sock` and `fc-gs.sock`). *Why:* FC is the contract authority — its peripheral and telemetry contracts define the message schema; clients connect to it.
- **The simulator is the postcard-rpc server on `sim-gs.sock`.** *Why:* sim-config and sim-control traffic stays off the FC entirely; decouples sim release cadence from FC release cadence.
- **The GS backend is a client on every link.** Two postcard-rpc client connections (`fc-gs.sock`, `sim-gs.sock`) and a REST server for the frontend.

postcard-rpc's `Server` type handles one connection at a time, so two focused server instances on FC are cleaner than one server fanning out to multiple clients.

### 8.3 Startup order

`xtask` is the orchestrator. `cargo xtask run-host`:
1. Builds all four binaries.
2. Spawns processes in dependency order: **FC first** (server on `fc-sim.sock` and `fc-gs.sock`), then **simulator** (listens on `sim-gs.sock`, connects to FC on `fc-sim.sock`), then **GS backend** (connects to both), then **GS frontend** (connects to GS backend over REST).
3. Multiplexes all four stdouts with role labels.
4. Tears them all down on Ctrl-C.

A peer that is not yet listening when its client connects causes the client to fail immediately and the run aborts. There is no retry loop — the orchestrator is responsible for ordering.

### 8.4 Transport adapter

```
InterprocessWireTx  ←→  tokio::io::split (write half) of interprocess::local_socket::tokio::Stream
InterprocessWireRx  ←→  tokio::io::split (read half)  of interprocess::local_socket::tokio::Stream
```

- Lives in `proto/` (or a thin sibling crate).
- ~50 LOC. Everything else reuses existing postcard-rpc infrastructure.
- postcard-rpc owns the framing — no hand-rolled length prefix or type tag.
- After connect, both halves moved into separate tasks (one tx, one rx).
- Listener / connector identity is a startup-ordering detail; postcard-rpc server / client role is the only durable asymmetry. Once connected, both sides have a symmetric full-duplex byte stream.

### 8.5 Reconnect semantics

One socket per peer pair, atomic reconnect on peer restart:
- A peer crashing closes the socket. The surviving peer drops its connection state and **the scenario ends**. There is no partial reconnect; restarting requires a fresh process.
- This matches sim run-lifecycle semantics: one peer crashing is treated as the run terminating, not as a transient fault to recover from.
- Two-socket topology (one per direction) was rejected: it adds partial-reconnect state with no compensating benefit at sensor-tick traffic volumes.

### 8.6 Topic vs Endpoint

- **Topics** for periodic, fire-and-forget data: sensor data (sim → FC), telemetry records (FC → GS), sim status (sim → GS).
- **Endpoints** for request/response or commanded actions: deployment (FC → sim), arm trigger (GS → sim), ignition (GS → sim), GS commands to FC, sim-control, config-hash handshake.

postcard-rpc demultiplexes both over the single per-pair stream.

### 8.7 PIL — same contract, different transport

PIL runs the FC binary on the production MCU; the simulator stays on host. The FC ↔ Sim link uses **the same postcard-rpc Topics and Endpoints as `fc-sim.sock`** but rides over USB instead of an interprocess socket. The same on-board postcard server multiplexes telemetry, GS commands, and simulator-fed sensor data over one USB wire — there is no second server. The GS backend ↔ Sim link (`sim-gs.sock` analogue) stays host-local in PIL because both endpoints run on host.

### 8.8 HW build exclusions (application-level)

`fc-sim.sock` and `sim-gs.sock` are absent at the application level in HW builds — not merely uncreated at runtime, but not compiled in at all. The `impl_host` feature gates every code path that opens or listens on `fc-sim.sock`; disabling it is sufficient to exclude all host-IPC code from the firmware binary. `sim-gs.sock` is a simulator-process concern that does not exist in any FC firmware artefact. HW builds also exclude simulator endpoints (gated behind `simulator-endpoints` feature on `proto/`).

This is an application-level constraint, not a named-pipe or OS-level one: the OS socket path is irrelevant because the code to create or connect to it is never compiled.

---

## 9. Observability

### 9.1 Framework

- **`tracing` + `tracing-subscriber`** for span emission and instrumentation across all host processes.
- **OpenTelemetry (OTEL) export** for cross-process correlation. Each process exports spans to a host-local OTEL collector / file sink, sharing trace IDs across the three.
- `#[instrument]` on async fn boundaries that matter (sensor parse, FSM transitions, scripted-event firing, postcard-rpc handlers).

### 9.2 Cross-process trace correlation

Trace ID propagation rule:
1. The trace ID for a scenario is **established by GS** when the operator initiates a run (Start command on `sim-gs.sock`).
2. GS embeds the trace ID in lifecycle and command messages it sends. Sim and FC pick it up on receipt.
3. Spans on each process attach to the propagated trace ID, so all three processes' spans share a root.
4. Spans within a process are children of whichever incoming-message span triggered the work.

This makes the operator click — "Start" — the natural root of every per-scenario trace tree. *Why GS as root:* the operator's intent is the causal entry point for everything that follows.

### 9.3 Panic capture

Every host process must:
- Install a panic hook before any work starts.
- The hook records the panic location, message, and current span context via `tracing` at `error` level.
- The hook ensures the tracing subscriber's exporter has flushed to its sink before the process exits.

A panic that exits without flushing is a bug — fix the panic hook, do not work around it.

*Why:* the postcard-rpc disconnect that follows a panic gives the surviving peer (and operator) almost no information by itself. The trace + flushed log is what tells them *why*.

### 9.4 Phasing

| Phase | Scope | When |
|---|---|---|
| 1 | Host targets only — FC-host, simulator, GS backend | Now (alongside the binary split) |
| 2 | Embedded targets — FC firmware on `cross-esp32-s3` and `cross-nucleo-f413zh` | After phase 1 is in production use |

**Phase 2 embedded approach.** `tracing` is viable under `no_std` — spans emitted via `defmt` or a custom transport. A companion host-side process reads the embedded trace stream and forwards it into the host OTEL pipeline so embedded spans land in the same trace tree. Specific embedded transport (defmt-rtt, USB serial postcard, etc.) is decided in Phase 2 against measured behaviour, not now. If `tracing` under `no_std` proves impractical at that point, a fallback plan is reassessed.

### 9.5 Sim-side logging

The simulator logs its own state (physics state per tick, force-event creation / expiry, scripted-event firing) using `tracing` at `debug` / `info` levels. Spans use the per-scenario trace ID so they correlate with FC and GS spans for the same run.

This is **separate from** FC storage logging (`Record`s to flash / SD via `FileSystem` for post-flight analysis). Both exist; they serve different audiences:
- Storage records → post-flight playback via GS.
- Tracing spans → live diagnosis, panic context, performance profiling.

---

## 10. Crash & disconnect policy

### 10.1 Per-component crash matrix

| Crashed | FC behaviour | Sim behaviour | GS behaviour |
|---|---|---|---|
| **GS backend** | Continue. Log comms failure. Telemetry buffers fill until backpressure. | Continue. Log comms failure. | — |
| **FC** | — | Shut down. Run depends on FC. | Wait for reconnect. Mark FC panel red with last-known-state stale. |
| **Simulator** | Shut down. FC blocks waiting on sensor data. | — | Wait for reconnect. Mark Sim panel red with last-known-state stale. |

The asymmetry is deliberate:
- **FC and Sim are run-lifecycle peers.** Either dying ends the run, so the survivor exits. *Why:* keeping a half-stack alive invites stale state that will mislead the next attempt.
- **GS is observational.** Its absence is degraded but not invalidating; FC and Sim keep running so a reconnecting GS picks up live data.

The matrix above is HOST-mode focused. **PIL inherits the same FC ↔ Sim policy over USB; the GS backend ↔ Sim link in PIL is host-local and follows the same rules as HOST.** Per-mode summary:

| Mode | FC crashes | Sim crashes | GS crashes |
|---|---|---|---|
| **HW** | Production reset / watchdog. No software shutdown path; recovery is hardware-level. | n/a | Comms-failure trace; FC keeps flying. |
| **HOST** | Sim shuts down (no peer); GS marks FC panel red with last-seen + last-known data. | FC shuts down (no input); GS marks Sim panel red. | FC and Sim continue; log comms failure. |
| **PIL** | MCU reset; sim on host shuts down on USB drop. | FC firmware blocks waiting for sensor data; restart needed. | FC and Sim continue; log comms failure. |

### 10.2 GS operator UX during disconnect

When GS loses one of its peer connections (`fc-gs.sock` or `sim-gs.sock`):
1. **Within one UI refresh:** affected panel turns **red**.
2. **Last seen:** `Last seen: <duration> ago` adjacent to the red indicator.
3. **Last known state:** the values from the last successfully-received message remain visible, dimmed, with a stale-indicator badge. The UI does **not** go blank.
4. **Operator controls:** Restart and Shutdown buttons available; **no automatic reconnect retry** — operator decides whether the run is recoverable.

The frontend UI surfaces the same red/last-seen state via REST poll on the backend.

*Why preserve last-known state:* operators routinely want to look at the last-good telemetry to diagnose what happened. A blanked UI hides exactly the data the operator needs.

*Why no auto-retry:* a peer that crashed once will likely crash the same way on restart; auto-retry hides root causes and produces noisy partially-progressed runs. Manual restart forces the operator to acknowledge.

### 10.3 Reconnect

- For FC ↔ Sim: neither survives, so reconnect is moot — both are restarted by the orchestrator.
- For GS ↔ FC and GS ↔ Sim: GS waits indefinitely for a fresh connection. There is no internal timeout. The operator decides when to give up.

---

## 11. Known limitations (accepted)

| Limitation | Decision | Why |
|---|---|---|
| Force direction not implemented | Deferred; data structure will refactor when added | Premature complexity; current scenarios are 1D parabolic |
| Force expiry only at tick boundary | By design; documented | Tick rate fast enough that pro-rating is below noise |
| Embedded-side tracing not implemented | Phase 2; host-only for now | Host tracing covers the simulator side |
| Sim/HW strictly one-or-the-other | By design | Hybrid mode adds complexity with no current need |
| No mid-tick force pro-rating | By design | Tick rate high enough to make this negligible |

---

## 12. Open questions

- **Sim physics-state reset on FC restart.** When the FC reconnects to a still-running sim mid-scenario, does the sim rewind physics to t=0 or keep running? Tracked in [`../TODO.md`](../TODO.md). In current model, FC ↔ Sim survivor exits on peer crash, so mid-scenario reconnect is not reachable; this question only matters if that policy changes.
- **Telemetry / sensor publish rate.** 10 Hz current target as estimate; actual rate is a tuning parameter, not an architectural commitment.
- **Trace-ID propagation mechanism in postcard-rpc** — dedicated header, message field, or per-Endpoint convention. To be settled in implementation.
- **OTEL collector destination on host** — file sink (likely) or local collector process.
- **Incremental testing strategy during binary-split migration** — which tests stay on the in-process `Signal` harness, which migrate to IPC clients first, how to avoid a regression gap. See [`../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md`](../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md) deferred work.

---

## 13. How a HOST scenario flows (walk-through)

1. **Operator** edits the scenario config file held by the GS backend. GS computes its hash.
2. **Operator** runs `cargo xtask run-host`. xtask spawns FC, simulator, GS backend, GS frontend in dependency order.
3. **Simulator** loads the same config (via CLI path), validates every field, computes its hash, listens on `sim-gs.sock`. It connects to FC on `fc-sim.sock`.
4. **GS backend** connects to FC on `fc-gs.sock` and to Sim on `sim-gs.sock`. On the sim socket, it receives the sim's config hash and verifies it against its own. Mismatch → run rejected, operator told.
5. **Operator** issues `Start` on `sim-gs.sock`. GS attaches a fresh trace ID; the simulator enters runtime phase.
6. **Simulator** ticks physics, publishes sensor data on `fc-sim.sock`. **FC** consumes it through `Sensor::parse_new_data`, runs FSM transitions, publishes telemetry Records on `fc-gs.sock`.
7. **Operator** clicks "Motor Ignition" in the frontend → REST → backend → Endpoint on `sim-gs.sock` → simulator spawns a thrust `ForceEvent`. The FC sees the consequences (acceleration on IMU, altitude rising on barometer) without ever knowing where the force came from.
8. **Operator** sees: live telemetry from FC, sim phase indicator, force-event list, scenario-level trace correlated across all three processes.
9. If anything panics, the panic hook flushes traces before exit. Surviving processes follow the crash policy: GS waits with last-known data and a red panel + last-seen timestamp; FC and Sim shut down because the run is over.

---

## 14. Where things live

```
docs/
├── software/
│   ├── spec.md                      ← this document (the consolidated spec)
│   └── README.md                    ← index
└── ADR/
    └── ADR-001-…                    ← Why postcard-rpc over alternatives (rationale)

code/
├── flight-computer/                 ← FC library crate + crate-level README
├── simulator/                       ← Simulator crate
├── ground-station-backend/          ← GS backend crate
├── proto/                           ← Wire-format crate
└── xtask/                           ← Orchestrator
```

---

## See also

- [`../README.md`](../README.md) — `docs/` scope; architecture vs detailed-design split.
- [`../how-we-work.md`](../how-we-work.md) — spec / ADR policy and traceability rules.
- [`../REQUIREMENTS.md`](../REQUIREMENTS.md) — the requirements (`[SW-*]`) this spec implements.
- [`../ROADMAP.md`](../ROADMAP.md) — implementation milestones for the binary split.
- [`../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md`](../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md) — *why* postcard-rpc + binary split.
- [`../../code/README.md`](../../code/README.md) — workspace map and crate roles.
