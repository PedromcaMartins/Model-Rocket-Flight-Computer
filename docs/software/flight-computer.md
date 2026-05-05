# Flight computer library

The FC library (`code/flight-computer/`) is the software core of the rocket. This document captures its design goals, the constraints that shape it, and the properties any contributor must preserve.

## Goals

- **One library, three deployment targets.** The same FC code runs on the production MCU (HW), on the production MCU with a simulator-fed USB interface (PIL), and on a host machine paired with a separate simulator process (HOST). No mode-specific branches in the FC core.
- **Peripheral-agnostic.** The FC library never imports a driver, a simulator type, or a transport crate. All hardware interaction goes through traits defined in `interfaces/`; the implementation behind each trait is supplied by the binary that links the library.
- **Runtime-agnostic async.** The library uses `async fn` throughout but does not depend on a specific async executor or runtime. Embassy provides the executor on HW and PIL; Tokio provides it on host. The FC library is oblivious to the difference.
- **Architecture-agnostic.** The library compiles for RISC-V (ESP32), ARM Cortex-M (STM32), and x86/x64 (host). It must remain `no_std`-compatible for embedded targets.
- **Single wire vocabulary.** All telemetry, commands, and simulator data use the types defined in `proto/` and the postcard-rpc endpoint/topic definitions declared there. A ground-station client built against those definitions works in HW, PIL, and HOST mode without change.

## Non-goals

- This is **not a framework** and not a domain-level reusable library. It is the flight software for this specific rocket. Generalising it to other rockets is explicitly out of scope.
- Real-time scheduling guarantees in software modes. Host execution is untimed.
- Providing its own async executor or HAL. Those are supplied by the binary.

## The trait system

```
code/flight-computer/src/interfaces/
    sensor.rs           — Sensor<Data, Error>: periodic data source
    arming_system.rs    — ArmingSystem: waits for arm signal
    deployment_system.rs — DeploymentSystem: fires parachute actuator
    led.rs              — Led: on / off / toggle status indicator
    filesystem.rs       — FileSystem: append-only record storage
```

Each trait has multiple implementations:

| Implementation | Feature flag | Used in |
|---|---|---|
| `impl_embedded` | `impl_embedded` | HW — real hardware drivers via `embedded-hal` |
| `impl_software` | `impl_software` | PIL — postcard-rpc client calls over USB to the host simulator |
| `impl_host` | `impl_host` | HOST — postcard-rpc client calls over interprocess local socket to the simulator binary |

No FC core module imports any of these implementations. The binary that links the library picks an implementation at compile time by enabling the matching feature.

## Async runtime dependency

The library calls `.await` but never spawns tasks or creates executors. All `async fn` trait methods are runtime-neutral. The only temporal primitive used internally is `embassy_time::Timer` / `Ticker`, which is driven by a platform-supplied `embassy-time` driver.

**Known platform dependencies** (must be satisfied by the linking binary):
- `embassy-time` HAL driver — provides the time source for `Ticker` and timeouts.
- `critical-section` implementation — required by `embassy-sync` primitives used in inter-task channels.

On host these are provided by Tokio + the `embassy-time-driver-std` (or equivalent). On embedded they are provided by the target BSP crate.

## postcard-rpc integration

The FC library communicates with the outside world (ground station, and in HOST/PIL mode with the simulator) exclusively through postcard-rpc. The endpoint and topic definitions live in `proto/` and are shared across all three modes:

| Mode | Transport medium | Same Topics/Endpoints? |
|---|---|---|
| HW | USB / radio | ✓ |
| PIL | USB (same wire as GS) | ✓ |
| HOST | interprocess local socket (`fc-sim.sock`) | ✓ |

The transport adapter is the only thing that differs. On HOST the adapter is `InterprocessWireTx` / `InterprocessWireRx` (wrapping `interprocess::local_socket::tokio::Stream`); on HW/PIL it is the USB serial codec already in use. See [ADR-001](../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md) for the decision.

## See also

- [`deployment-modes.md`](deployment-modes.md) — the three deployment topologies and what crosses the wire in each.
- [`fc-simulator-interface.md`](fc-simulator-interface.md) — the peripheral-trait contract and its postcard-rpc implementation.
- [ADR-001](../ADR/ADR-001-fc-simulator-postcard-rpc-ipc.md) — binary split and postcard-rpc transport decision.
- [`../../code/flight-computer/src/interfaces/`](../../code/flight-computer/src/interfaces/) — trait definitions.
