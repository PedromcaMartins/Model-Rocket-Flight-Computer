# flight-computer-host — spec

- **Status:** implementation
- **Implements:** `docs/software/spec.md` §8 (Host IPC), §2 (HOST topology)

## Role

The HOST deployment mode binary. Starts flight computer (FC) as a user-space
process on a development machine, accepting simulator sensor data on one
local socket (`fc-sim.sock`) and GS telemetry/commands on another
(`fc-gs.sock`).

## Socket topology

```
fc-sim.sock  ─── Simulator (peripheral data: sensors, arm, deploy, LED)
fc-gs.sock   ─── GS backend  (telemetry topics, command endpoints)
```

Both sockets use `interprocess::local_socket` with `GenericNamespaced`,
which abstracts the OS difference:
- Linux   → abstract namespace socket (no filesystem artifact)
- Windows → `\\.\pipe\*` named pipe

**Why local sockets over TCP:** no networking overhead, same-machine
enforcement, single binary for Linux and Windows without `#[cfg]`.

## Startup sequence

1. Bind both listener sockets (allows clients to connect immediately).
2. Accept simulator connection (blocking — waits until simulator connects).
3. Start flight computer with a **GS backend factory** — each call accepts
   one GS connection, retrying on transient errors. The factory is invoked
   each time the GS subsystem loop attempts a (re)connect; a missing or
   restarting GS never blocks the FC ↔ Sim loop.
4. Inside `start_host_flight_computer`, the sim server runs directly as a
   `postcard_server_task`; the GS factory is called on demand by the GS
   subsystem loop.

Sim accept is blocking because the FC has nothing to do until sensor data
arrives. GS accept is deferred: the FC operates without a GS connection and
will accept one whenever it connects (factory pattern). The orchestrator
(`xtask`) spawns processes in the order: `GS backend → simulator → FC-host`,
but FC-host independently handles whichever peer arrives first.

## Dispatch layout

Two separate `define_dispatch!` invocations in `dispatch.rs`:

| Dispatch | Socket | Endpoints | Topics in | Topics out |
|---|---|---|---|---|
| `SimDispatch` | `fc-sim.sock` | (none) | `TOPICS_SIM_IN_LIST`: altimeter, GPS, IMU, arm | `TOPICS_SIM_OUT_LIST`: deploy, LEDs |
| `GsDispatch` | `fc-gs.sock` | `PingEndpoint`, `GlobalTickHzEndpoint` | `TOPICS_GS_IN_LIST` (empty) | `TOPICS_GS_OUT_LIST`: records |

All handlers are `blocking` — no async/spawn dispatch needed. The
`ChannelWireSpawn` + `tokio_spawn` from postcard-rpc's `test_channels`
are used for the spawn infrastructure; they are never called at runtime
but will correctly spawn futures if non-blocking handlers are added later.

Both dispatches share the same `Context` (currently empty, holds no
state).

## Config

All compile-time constants (`Config` unit struct per
`flight-computer/src/config.rs` pattern):

No runtime config loading — values are fixed at compile time.

## Logging

Initialized in `main` by `logging::init_tracing()`. Writes structured JSON
logs to `logs/<unix-timestamp>/`:

```
logs/<ts>/
├── info.json       # INFO level
├── debug.json      # DEBUG level
├── warn.json       # WARN level
├── error.json      # ERROR level
├── trace.json      # TRACE level
└── log.json        # all levels combined
```

Stdout output: human-readable with file, line, thread, target. Level
filtered by `RUST_LOG` env var (default INFO).

## Panic hook

Captures panic location, message, and span context via `tracing::error!`
before forwarding to the default panic hook.
