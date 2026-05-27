# simulator — detailed design

Crate-level detailed design for the standalone simulator binary. Architectural
constraints (the FC ↔ simulator interface) live in `docs/software/spec.md`; this
document covers the crate's internal architecture.

> **Living document.** Update this spec in the same change that alters the
> design. Drift is a bug (see `AGENTS.md §1`).

---

## 1. Scope

The simulator is an independent process that closes the sensor → FC FSM →
deployment loop over postcard-rpc, with no ground-station process present.

**In scope**

- postcard-rpc **client** connecting to the FC-host **server** on `fc-sim.sock`.
- 3D physics (kinematic attitude, multi-axis forces, drag) using the `spatial` crate for coordinate frame safety.
- Scripted scenario from a compile-time config struct.
- Read-only ratatui TUI.
- Structured tracing (per-level JSON + stdout) and a panic hook.
- Graceful shutdown on Ctrl-C.
- Two binaries — `host` (interprocess socket) and `pil` (USB) — over one shared
  library.

**Out of scope**

| Deferred | Reference |
|---|---|
| `sim-gs.sock` server, GS lifecycle / trigger endpoints, config-hash handshake | `docs/ROADMAP.md`, `proto/` |
| Two-phase Setup→Runtime lifecycle, Restart | `docs/ROADMAP.md` |
| Config from TOML file + validation + hashing | `docs/ROADMAP.md` |
| Interactive (keyboard) TUI controls | `docs/ROADMAP.md`, crate `README.md` |
| Full 6-DOF rotational dynamics (angular inertia, aerodynamic moments, thrust torque) | `docs/ROADMAP.md` |

See `docs/ROADMAP.md` for the full project plan and `README.md` for
crate-specific deferred features.

---

## 2. Crate structure — library + two binaries

The simulator is a **library crate** holding all shared logic, with two thin
binary entry points that differ **only** in how they construct the postcard-rpc
client transport. Everything downstream of the client is transport-agnostic.

```
code/simulator/
├── spec.md                  ← this document
├── README.md                ← crate overview
├── src/
│   ├── lib.rs               ← run_simulator(client, cancel, tui_cancel) + module decls
│   ├── config.rs            ← SimulatorConfig (physics) + Config (infrastructure)
│   ├── connect.rs           ← transport connect with exponential backoff
│   ├── flight_computer.rs   ← publish sensor topics / subscribe actuator topics + FcCommand
│   ├── logging.rs           ← per-level JSON + stdout + panic hook
│   ├── scripted.rs          ← timed script: arm → wait → ignite
│   ├── types.rs             ← domain types (ForceEvent, FcCommand, shared snapshots)
│   ├── physics/
│   │   ├── mod.rs           ← run_physics_loop: timing + integration
│   │   ├── engine.rs        ← PhysicsEngine (state machine + force integration)
│   │   └── state.rs         ← PhysicsState + From<> sensor conversions
│   └── tui/                 ← read-only ratatui TUI
│       ├── mod.rs           ← blocking bridge + event loop
│       ├── render.rs        ← layout and panel rendering
│       ├── actuators.rs     ← LED status display
│       └── logs.rs          ← colorized log viewer
└── src/bin/
    ├── host.rs              ← interprocess socket → connect_with_retry → run_simulator
    └── pil.rs               ← USB (deferred)
```

### 2.5 — The `spatial` crate (host-side frame conversions)

A new workspace member at `code/spatial/` wraps [`sguaba`](https://docs.rs/sguaba/latest/sguaba/)
to provide type-safe reference frame conversions. It is used **only** by host-side
crates (simulator, ground-station) — never by `proto` or the FC library.

```
code/spatial/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs        — re-exports sguaba, nalgebra, uom, serde
    ├── frames.rs     — system!(struct LaunchNed using NED)
                        system!(struct RocketFrd using FRD)
                        type aliases: NedPosition, NedVelocity, etc.
    └── convert.rs    — FrameConversions: NED↔WGS84, NED↔ECEF, body-frame rotation
                        — converts spatial types (nalgebra 0.34, uom 0.38) to
                          proto wire types (nalgebra 0.33, uom 0.37)
                        — single unsafe call site: RigidBodyTransform::ecef_to_ned_at
```

**Dependency story.** Two independent nalgebra+uom versions coexist:

| Layer | nalgebra | uom | Used by |
|---|---|---|---|
| `spatial` / `sguaba` | 0.34 | 0.38 | Host-side: sim, GS, any frame-conversion code |
| `proto` / `flight-computer` | 0.33 | 0.37 | FC firmware, wire format (no_std) |

The `spatial` crate provides explicit per-field conversion functions between the two.
No `From` blanket impls across crate versions.

**Frame definitions:**
- `LaunchNed` — NED (North-East-Down) tangent plane at the launchpad. The principal
  simulation frame. Origin is the launchpad position.
- `RocketFrd` — FRD (Front-Right-Down) body frame. +Front is the rocket nose,
  +Right is starboard, +Down is through the belly.

**FrameConversions** pre-computes the `RigidBodyTransform::ecef_to_ned_at` for the
launchpad WGS84 position. Key methods:

| Method | Direction | Purpose |
|---|---|---|
| `ned_to_gps` | Spatial → Proto | Simulator: NED position → GPS lat/lon |
| `ned_to_altitude` | Spatial → Proto | Simulator: NED down → MSL altitude |
| `ned_accel_to_body` | Spatial → Proto | Simulator: NED acceleration → body-frame IMU |
| `gps_to_ned` | Proto → Spatial | GS: received GPS → NED offset from launchpad |

`lib.rs` exposes a single transport-generic entry point:

```rust
pub async fn run_simulator(client: PostcardClient, cancel: CancellationToken, tui_cancel: CancellationToken);
```

Both binaries build the `PostcardClient`, install logging, then call
`run_simulator`. The library never knows which transport it runs over.

**Config convention.** Two structs by design:

- `SimulatorConfig` — physics parameters (fn getters, const-first; `fn` only
  when const arithmetic is impossible for uom types, never takes `self`).
- `Config` — infrastructure parameters (plain `pub const`).

---

## 3. Transport — simulator is the client

Per `docs/software/spec.md`, the **FC-host binds and accepts** on `fc-sim.sock`;
the **simulator connects as the postcard-rpc client**.

| Binary | Transport | Construction |
|---|---|---|
| `host` | interprocess local socket `fc-sim.sock` | connect a stream, wrap in a client wire, build `PostcardClient` |
| `pil`  | USB | `PostcardClient::try_new_raw_nusb(..)` — deferred |

---

## 4. Wire contract

The simulator publishes sensor topics (`TOPICS_SIM_IN_LIST` in `proto`) and
subscribes actuator and flight-state topics (`TOPICS_SIM_OUT_LIST` +
`SimFlightStateTopic`; canonical list in `proto/src/lib.rs`).

Sensor messages are derived from `PhysicsState` via `From<PhysicsState>`.
Actuator status and LED states are aggregated into a shared snapshot for the TUI.

**FlightState feedback** creates a control loop: scripted waits for
`FlightState::Armed` from the FC before triggering ignition.

---

## 5. Closed-loop event flow

```
┌──────────────┐
│   scripted   │  — one-shot, emits domain commands/triggers
└──┬───────┬───┘
   │       │
   │  FcCommand::Arm       ForceEvent::MotorThrust
   │       │
   ▼       ▼
┌────────────────────────┐          ┌───────────────────────────┐
│    fc_client task      │◀─subscribe─  FC-host (postcard-rpc) │
│  (publish / subscribe) │──publish─▶     (flight computer)    │
└─────────┬──────────────┘          └───────────────────────────┘
          │
          │   ForceEvent::Recovery  (when FC fires deployment)
          │
          ▼
┌──────────────────┐
│  physics engine  │  — 1 ms step, publishes PhysicsState @ 20 ms
└────────┬─────────┘
         │
         ├── PhysicsState (watch) → fc_client publishes sensors
         ├── PhysicsState (watch) → TUI displays state
         ├── ActiveForceEvent (ArcSwap) → TUI displays active forces
         │
         │   ArcSwap<SimActuatorSnapshot>
         ├── fc_client ← (actuator subscriptions) → TUI reads for display
         │
         │   FlightState (watch)
         └── fc_client ← SimFlightStateTopic → scripted waits for arm
```

**3D force composition.** All forces are expressed as 3-vectors in the `LaunchNed`
frame. Gravity is `[0, 0, m·g]`. Thrust and drag are computed in `RocketFrd` and
rotated to `LaunchNed` using the kinematic attitude quaternion. Recovery drag
opposes velocity in NED. See `physics/engine.rs`.

**Separation principle.** Scripted speaks only domain types (`FcCommand`,
`ForceEvent`). It never touches the postcard-rpc wire. fc_client translates
between domain types and postcard-rpc topics. Physics knows only
`ForceEvent` triggers and publishes `PhysicsState` via watch — it has no
knowledge of topics, the wire, or FC commands.

---

## 6. Shared state

Five channels and one shared atomic store connect the tasks. Each producer owns
the state it writes; readers subscribe or load lock-free.

| Data | Producer | Consumer(s) |
|---|---|---|
| `PhysicsState` (sensor snapshot) | physics engine (every 20 ms) | fc_client → publish, TUI → display |
| `FlightState` (FC status) | fc_client (from FC via `SimFlightStateTopic`) | scripted (arm confirmation) |
| `FcCommand` | scripted | fc_client (translate to `SimArmTopic`) |
| `ForceEvent` (physics triggers) | scripted + fc_client | physics engine (integrate) |
| `SimActuatorSnapshot` (LED + deployment) | fc_client (from FC subscriptions) | TUI (lock-free, ArcSwap) |
| Active force events | physics engine (derived each step) | TUI (lock-free, ArcSwap) |

No monolithic `SimState` — each channel carries exactly what its consumer needs.
The TUI reads three sources (PhysicsState, active forces, actuator snapshot)
independently.

---

## 7. Two rates

Physics integrates at `PHYSICS_TIME_STEP` (1 ms). Sensor messages are published
to the FC at `DATA_ACQUISITION_INTERVAL` (20 ms). A compile-time assertion
enforces that acquisition is no faster than the physics step.

---

## 8. Tasks, cancellation, resilience

`run_simulator` spawns one task per domain. Two cancellation tokens control
shutdown:

- **`cancel`** — stops physics, fc_client, scripted. Fired by Ctrl-C or FC
  disconnect cascade.
- **`tui_cancel`** — stops the TUI event loop. Fired by user `q`/Esc or Ctrl-C.

**Three shutdown paths:**

1. **User quits TUI** (q/Esc/Ctrl-C in TUI) → `tui_cancel` fires → TUI returns
   → `run_simulator` cascades to `cancel` → physics/fc_client/scripted stop →
   exit 0.
2. **Ctrl-C while TUI not focused** → handler fires both tokens → `tui_cancel`
   wins the biased select → cascade to `cancel` → all tasks stop.
3. **FC disconnects** → fc_client exits → `cancel` fires (physics/scripted
   stop) but `tui_cancel` is **not** fired. TUI keeps rendering with a
   "FC DISCONNECTED" banner until the user quits — this is **degraded mode**.

---

## 9. TUI panels

The ratatui TUI displays four panels, each reading from its own shared state channel:

| Panel | Source data | Purpose |
|---|---|---|
| **Physics** | `PhysicsState` watch | Live position, velocity, acceleration, sim time |
| **Actuators** | `SimActuatorSnapshot` (ArcSwap) | LED on/off/toggle state per component; deployment actuator status |
| **Active Forces** | `ActiveForceEvent` (ArcSwap) | Current force-event list with magnitudes and remaining durations |
| **Logs** | `LOG_BUFFER` ring | Colorised tail of the structured sim log |

Each panel updates independently at the data's natural cadence. The Actuators
panel is the only place that surfaces raw FC peripheral calls (LED state,
deployment) — GS receives these as distilled named status values in the
telemetry stream per spec.md §5.1.

## 10. Logging

Per-level JSON files + stdout, mirroring `flight-computer-host/src/logging.rs`,
with a panic hook that emits a tracing error before the default handler.

---

## 11. Connect retry

Both binaries retry their transport connect with exponential backoff so the
simulator can start before its peer (FC host or MCU). The retry loop checks
cancellation before every attempt and wraps each connect in a timeout.

When launched via `cargo xtask host`, the retry almost always succeeds on the
first attempt. The backoff is a safety net for standalone runs and USB
enumeration delays.
