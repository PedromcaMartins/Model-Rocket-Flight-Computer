# simulator — detailed design (M2.2 MVP)

Crate-level detailed design for the standalone simulator binary. Architectural
constraints (the FC ↔ simulator interface) live in `docs/software/spec.md`; this
document covers the crate's internal design and is updated on every change.

> **Living document.** Update this spec in the same change that alters the
> design. Drift is a bug (see `AGENTS.md §1`). Record scope/decision changes in
> §11.

---

## 1. Scope

M2.2 delivers the simulator as an independent process that closes the
sensor → FC FSM → deployment loop over postcard-rpc, with no GS process present.

**In scope**

- postcard-rpc **client** connecting to the FC-host **server** on `fc-sim.sock`.
- 1D parabolic physics (motor burn → coast → apogee → descent → landing).
- Scripted scenario from a compile-time `pub const` config struct.
- Minimal **read-only** ratatui TUI.
- Structured tracing (per-level JSON + stdout) and a panic hook.
- Graceful shutdown on Ctrl-C.
- Two binaries — `host` (interprocess socket) and `pil` (USB) — over one shared
  library.

**Out of scope** (deferred — see §11 and `docs/ROADMAP.md`)

| Deferred | To |
|---|---|
| `sim-gs.sock` server, GS lifecycle/trigger Endpoints, config-hash handshake | M3.3 |
| Two-phase Setup→Runtime lifecycle, Restart | M2.4 |
| Config from TOML file + validation + hashing | M2.4 |
| Interactive (keyboard) TUI controls | M2.4 |
| 3D physics (inclination, azimuth, attitude) | M2.3 |

---

## 2. Crate structure — library + two binaries

The simulator is a **library crate** holding all shared logic, with two thin
binary entry points that differ **only** in how they construct the postcard-rpc
client transport. Everything downstream of the client is transport-agnostic.

```
code/simulator/
├── spec.md                  ← this document
├── README.md                ← brief overview
├── src/
│   ├── lib.rs               ← run_simulator(client, cancel) + module decls
│   ├── config.rs            ← pub const SIM_CONFIG: SimulatorConfig
│   ├── logging.rs           ← per-level JSON + stdout + panic hook
│   ├── types.rs             ← TriggerEvent, SimActuatorData, ActiveEventSummary
│   ├── fc_client.rs         ← publish sensor Topics / subscribe actuator Topics
│   ├── scripted.rs          ← timed TriggerEvent emitter
│   ├── tui.rs               ← read-only ratatui view
│   └── physics/
│       ├── mod.rs
│       ├── engine.rs        ← integration + force composition
│       ├── state.rs         ← PhysicsState + From<PhysicsState> for sensors
│       └── events.rs        ← ActiveCommand (force events)
└── src/bin/
    ├── host.rs              ← interprocess Stream → HostClient on fc-sim.sock
    └── pil.rs               ← HostClient::try_new_raw_nusb (USB)
```

`lib.rs` exposes a single transport-generic entry point:

```rust
pub async fn run_simulator(client: PostcardClient, cancel: CancellationToken);
```

Both binaries build the `PostcardClient`, install logging, then call
`run_simulator`. The library never knows which transport it runs over.

**Reuse / removal.** Port `physics/{engine,state,events}` from the existing code
(integration math, force model, `From<PhysicsState>` sensor conversions are
correct and kept). Delete the channel-based `api/`, `runtime/`,
`scripted_scenario/` modules and the in-process `simulator_loop` — they were the
preliminary in-process design and are superseded by the postcard-rpc boundary.

---

## 3. Transport — simulator is the client

Per `docs/ROADMAP.md` and `flight-computer-host/src/main.rs`, the **FC-host
binds and accepts** on `fc-sim.sock`; the **simulator connects as the
postcard-rpc client**.

| Binary | Transport | Construction |
|---|---|---|
| `host` | interprocess local socket `fc-sim.sock` | connect a `Stream`, wrap in a client wire, build `HostClient` |
| `pil`  | USB | `HostClient::try_new_raw_nusb(..)` — mirror `ground-station-backend/src/bin/serial/main.rs` |

A thin `PostcardClient` wrapper (modelled on
`ground-station-backend/src/postcard_client.rs`) provides typed
`publish_sim_*` / `subscribe_sim_*` methods over `HostClient<WireError>`.

> **Open design item — client-side IPC wire.** `proto::ipc_adapter` currently
> provides only the **server** side (`InterprocessWireTx/Rx`,
> `interprocess_wire_from_stream`) used by FC-host. The `host` binary needs the
> **client** counterpart: a `HostClient` constructed over a connected
> interprocess `Stream` (length-prefixed framing identical to the server, a
> spawned RX pump, `WireError` error path). This adapter does not exist yet —
> it must be added (likely in `proto::ipc_adapter` behind `ipc-adapter`) before
> the `host` binary can connect. The `pil` path is unblocked (USB constructor
> exists). Tracked in §11.

---

## 4. Wire contract (from `proto`, `simulator-endpoints`)

The simulator **publishes** `TOPICS_SIM_IN_LIST` (client → server) and
**subscribes** `TOPICS_SIM_OUT_LIST` (server → client). Verified against
`code/proto/src/lib.rs`.

**Published by simulator (sensors / arming):**

| Topic | Message | Source |
|---|---|---|
| `SimAltimeterTopic` | `AltimeterData` | `From<PhysicsState>` |
| `SimGpsTopic` | `GpsData` | `From<PhysicsState>` |
| `SimImuTopic` | `ImuData` | `From<PhysicsState>` |
| `SimArmTopic` | `ActuatorStatus` | scripted `Arm` trigger |

**Subscribed by simulator (FC actuator outputs):**

| Topic | Message | Effect |
|---|---|---|
| `SimDeploymentTopic` | `ActuatorStatus` | `Active` → recovery force event into physics |
| `SimPostcardLedTopic` … `SimGroundStationLedTopic` (8 LEDs) | `LedStatus` | display only (TUI) |

`AltimeterData`/`GpsData`/`ImuData` conversions and `ActuatorStatus {Active,
Inactive}` / `LedStatus {On, Off}` already exist in `proto` and `physics/state.rs`.

---

## 5. Closed-loop event flow

```
scripted (compile-time schedule)        FC-host (postcard-rpc server)
        │                                        ▲   │
        │ TriggerEvent                           │   │ SimDeploymentTopic
        ▼                                  sensor│   ▼ Sim*LedTopic
  ┌───────────────┐  mpsc<TriggerEvent>  ┌───────┴──────────┐
  │ scripted task │ ───────────────────▶ │  fc_client task  │
  └───────────────┘                      │  publish/subscribe│
        │                                └───────┬──────────┘
        │ Ignition/Deploy                        │ Deploy (from SimDeploymentTopic)
        ▼                                        ▼
  ┌──────────────────────────────────────────────────────┐
  │ physics engine  (step @ physics_time_step)            │
  │  watch<PhysicsState>  watch<Vec<ActiveEventSummary>>  │
  └───────────────┬──────────────────────┬───────────────┘
                  ▼                       ▼
          fc_client (publish      TUI (read-only)
          @ data_acquisition)  +  watch<Vec<SimActuatorData>>
```

**`fc_client` is the routing hub for all trigger events.** `mpsc<TriggerEvent>`
flows scripted → fc_client; fc_client routes by variant:

- `Ignition` / `Deploy` → forwarded into a second `mpsc` into the physics
  engine (as force events: thrust or recovery force).
- `Arm` → fc_client publishes `SimArmTopic(ActuatorStatus::Active)` on the
  wire to FC. Not a physics force.

The deployment loop is closed through fc_client: physics publishes sensors →
FC FSM decides to deploy → FC publishes `SimDeploymentTopic(Active)` →
fc_client receives it (via subscription) → sends `TriggerEvent::Deploy` into
the physics mpsc → recovery force event applied.

Physics is fully decoupled from the wire — it only ever receives force triggers
(`Ignition`, `Deploy`), never knows about postcard topics or arming signals.

---

## 6. Shared state — parallel watch channels (no monolithic struct)

Each producer task owns exactly the channels it writes; readers subscribe.
No `Arc<RwLock<…>>`, no single `SimState` god-struct.

| Channel | Writer | Reader |
|---|---|---|
| `watch::Sender<PhysicsState>` | physics | fc_client, scripted, TUI |
| `watch::Sender<Vec<ActiveEventSummary>>` | physics | TUI |
| `watch::Sender<Vec<SimActuatorData>>` | fc_client | TUI |
| `mpsc::Sender<TriggerEvent>` | scripted | fc_client (routing hub) |
| `mpsc::Sender<TriggerEvent>` (physics-only) | fc_client | physics engine |

`fc_client` is the sole consumer of the scripted `mpsc`. It routes Arm → wire
publish; Ignition/Deploy → physics `mpsc`. fc_client also sends Deploy into
the physics `mpsc` when `SimDeploymentTopic(Active)` arrives from FC.

`PhysicsState` is the single source of truth for sensors; sensor messages are
derived on demand via `From<PhysicsState>`. `PhysicsState.time` carries sim time
— scripted/TUI read it from the `watch`; no separate time channel.

---

## 7. Types (`types.rs`)

```rust
enum TriggerEvent { Ignition, Arm, Deploy }   // Deploy also arrives from FC

enum SimActuatorData {
    Deployment(ActuatorStatus),
    PostcardLed(LedStatus),
    AltimeterLed(LedStatus),
    GpsLed(LedStatus),
    ImuLed(LedStatus),
    ArmLed(LedStatus),
    FileSystemLed(LedStatus),
    DeploymentLed(LedStatus),
    GroundStationLed(LedStatus),
}

struct ActiveEventSummary { /* kind + remaining duration, for TUI */ }
```

Descriptive names only — no `fn_*`-style prefixes (per task feedback).

---

## 8. Two rates

`config.rs` holds two durations as `pub const`:

- `PHYSICS_TIME_STEP` (e.g. 1 ms) — engine integration step.
- `DATA_ACQUISITION_INTERVAL` (e.g. 20 ms) — sensor publish cadence.

The engine integrates every step; sensors are published only on the acquisition
tick. A **compile-time** assertion enforces
`DATA_ACQUISITION_INTERVAL >= PHYSICS_TIME_STEP`
(`const { assert!(..) }`), since publishing must be no faster than the sim step.

---

## 9. Tasks, cancellation, resilience

`run_simulator` spawns one task per domain with `tokio::spawn` (not a single
`select!`), each holding a clone of a `tokio_util::sync::CancellationToken`:

| Task | Lifetime | Notes |
|---|---|---|
| physics loop | spawned, cancellable | step @ `PHYSICS_TIME_STEP`; publishes state @ `DATA_ACQUISITION_INTERVAL` |
| fc_client | spawned, cancellable | publish sensors; subscribe actuator/LED topics |
| scripted | spawned, cancellable | sleeps to scheduled offsets, emits `TriggerEvent` |
| TUI | spawned, cancellable | restores terminal on cancel |

**Shutdown.** Ctrl-C → `cancel.cancel()` → each task observes
`cancel.cancelled()` at the top of its loop, cleans up (TUI restores terminal,
client flushes), then exits; `main` joins handles and exits 0.

**Resilience (per task decision).** The `fc-sim` pipe is **critical** — if it
breaks (`ConnectionClosed`), the simulator **panics**: FC ↔ Sim desync is
unrecoverable in the MVP. The panic hook records the cause to the JSON logs
before exit. (Reconnect/restart handling is M2.4.)

> **Cross-crate dependency.** `flight-computer/src/tasks/simulation.rs`
> `postcard_sim_task` must likewise **panic on error** rather than log-and-retry,
> so both ends fail fast on desync. Tracked in §11; not part of this crate.

---

## 10. Logging

Mirror `flight-computer-host/src/logging.rs` exactly for cross-binary
consistency: non-blocking `tracing_appender` writing per-level JSON files
(`info/debug/warn/error/trace.json` + combined `log.json`) under
`logs/<timestamp>/`, plus a stdout layer, plus `install_panic_hook()` that
emits `tracing::error!(%info, "process panicked")` before the default hook.
Both binaries call `install_panic_hook()` then `init_tracing()` first thing.

---

## 11. Status & change log

| Item | Status |
|---|---|
| Spec authored | Done |
| Client-side IPC wire adapter in `proto::ipc_adapter` (§3) | **Open — blocks `host` binary** |
| lib + `host`/`pil` bin skeleton | Not started |
| Port `physics/*`, delete `api`/`runtime`/`scripted_scenario` | Not started |
| `fc_client`, `scripted`, `tui`, `types`, `logging`, `config` | Not started |
| `flight-computer` `postcard_sim_task` panic-on-error (§9, cross-crate) | Not started |

**Decisions captured from planning (`task.md`):** clean rewrite alongside
postcard-rpc; compile-time `pub const` config (no CLI/TOML in MVP); lib + two
bins differing only in transport; two explicit rates with compile-time
assertion; parallel `watch` channels, no monolithic `SimState`; `From<PhysicsState>`
for sensors (no `SharedSensors`); `SimActuatorData` enum for actuator state;
`tokio::spawn` per domain + `CancellationToken`; Ctrl-C graceful shutdown;
`fc-sim` break → panic both ends; descriptive names (no `fn_*`).
