# ADR-001: Split host build into per-role binaries with postcard-rpc as unified RPC layer

- **Status:** Accepted (with resolved follow-ups and deferred work — see below)
- **Date:** 2026-05-05

## Context

Today the HOST deployment mode is a single binary that wires the flight-computer (FC) library, the simulator, and the ground-station backend together with in-process Embassy channels and direct library calls. This causes three problems:

- The simulator and the FC cannot be run independently. `cargo run -p simulator` and `cargo run -p flight-computer` exist as crates but cannot talk to each other.
- PIL (FC firmware on the prod board, simulator on host) has no implementation. It needs the same simulated-peripheral surface the host build uses, but routed over USB to the MCU.
- The FC library has to drag the simulator's wiring along even when the consumer doesn't want it.

The simulator drives more than just sensors. The FC's peripheral surface is multi-trait: [`Sensor`](../../code/flight-computer/src/interfaces/sensor.rs), [`ArmingSystem`](../../code/flight-computer/src/interfaces/arming_system.rs), [`DeploymentSystem`](../../code/flight-computer/src/interfaces/deployment_system.rs), [`Led`](../../code/flight-computer/src/interfaces/led.rs). The [`FileSystem`](../../code/flight-computer/src/interfaces/filesystem.rs) is the only peripheral with a real I/O even in HOST and PIL — every other peripheral has a simulated implementation. Any IPC scheme must carry all of them, in both directions.

## Options considered

- **Stay in-process (status quo).** Cheapest. Keeps everything blocking-fast in tests. Does not solve PIL, does not let the simulator run as a separate binary, and forces the simulator wiring into every FC consumer.
- **Single transport for everything (one wire format on host and over USB).** Uniform. But the host case wants a host-local IPC primitive (Unix sockets / named pipes) and the USB case wants a framed serial codec; sharing one stack means picking one and adapting it to the other, both of which lose properties.
- **One RPC framework (postcard-rpc), two transport media.** `postcard-rpc` is used for both the FC ↔ simulator link on host (over [`interprocess`](https://crates.io/crates/interprocess) local sockets) and the FC ↔ ground-station link (over USB / radio). Both reuse the `proto/` types and the same Topic/Endpoint primitives. The transport medium swaps; the RPC model and message vocabulary do not. PIL becomes expressible by reusing the same postcard-rpc server the ground station already talks to, with no second server or second link. Deployment on HW does not include the simulator endpoints, removing overhead.

## Decision

Adopt option 3. Concretely:

1. **Split HOST into four binaries** — flight computer, simulator, ground-station backend, ground-station frontend — each shipped from its own crate. **`xtask` is the orchestrator**: `cargo xtask run-host` builds all four, spawns them as OS processes in startup order (FC first — it is the postcard-rpc server and must be listening before any client connects; simulator and GS backend connect after), multiplexes their stdout with role labels, and tears them all down on Ctrl-C.
2. **The FC is the postcard-rpc server on every link it participates in.** On host it runs two server instances over [`interprocess::local_socket::tokio`](https://docs.rs/interprocess/latest/interprocess/local_socket/tokio/index.html) — one per client — forming a three-socket triangle:

   | Socket | Server | Client | Traffic |
   |---|---|---|---|
   | `fc-sim.sock` | FC | Simulator | sensor Topics (sim→FC), deployment/LED Topics (FC→sim) |
   | `fc-gs.sock` | FC | GS backend | telemetry Topics (FC→GS), command Endpoints (GS→FC) |
   | `sim-gs.sock` | Simulator | GS backend | runtime config Endpoints and physics-status Topics (GS↔sim) |

   postcard-rpc's `Server` type handles one connection at a time; two focused server instances on FC are cleaner than one server fanning out to multiple clients. The GS backend holds two client connections; FC and Simulator each run two server instances (one each). A thin transport adapter (`InterprocessWireTx` / `InterprocessWireRx`) wraps the `tokio::io::split` halves of the `Stream` and implements postcard-rpc's `WireTx` / `WireRx` traits. Listener vs connector is purely a startup-ordering concern; once connected, both sides have a symmetric full-duplex byte stream and the postcard-rpc server/client role is the only asymmetry.
3. **FC ↔ ground-station also uses postcard-rpc**, over USB / radio. Both host-local and GS links now share the same RPC framework and message vocabulary; only the `WireTx`/`WireRx` implementation differs.
4. **Wire types live in [`proto/`](../../code/proto/)** and are shared across all links. The `InterprocessWireTx`/`WireRx` adapter lives in `proto/` (or a thin sibling crate) so neither the FC library nor the simulator crate depends on the other.
5. **Two simulated-peripheral implementations**, both implementing the FC's peripheral traits (`Sensor`, `ArmingSystem`, `DeploymentSystem`, `Led`):
   - **Host simulated peripherals** — postcard-rpc client calls over the interprocess socket to the simulator.
   - **PIL simulated peripherals** — postcard-rpc client calls over USB to the host-side simulator, reusing the same server the ground station talks to.
6. **The three-socket triangle satisfies the N-way traffic requirement.** Each link carries only the traffic that belongs to it: FC ↔ Sim for peripheral data and actuation, FC ↔ GS for telemetry and flight commands, Sim ↔ GS for runtime configuration and physics status. No link is a proxy for another. Ignition (GS operator → sim via `sim-gs.sock`) and parachute deployment (FC → sim via `fc-sim.sock`) flow on separate links without coupling.
7. **Test ergonomics are deferred.** Reworking unit tests that today write to a static `Signal` is a future problem; it does not block the binary split.

## Open follow-ups (Resolved)

- **Connect direction under bidirectional traffic.** One socket per peer pair; the postcard-rpc server listens, the client connects. FC listens on `fc-sim.sock` and `fc-gs.sock`; Simulator listens on `sim-gs.sock`. Both sides immediately call `tokio::io::split()` to get owned read and write halves moved into separate tasks. postcard-rpc owns the framing — no hand-rolled length prefix or type tag is needed. Head-of-line blocking is not a concern at sensor-tick frequencies. Two-socket topology (one per direction) was rejected: a single socket provides atomic reconnect on peer restart, matching run-lifecycle semantics (one peer crashing ends the scenario), and avoids partial-reconnect state with no compensating benefit at this traffic volume. Listener vs connector is symmetric at the transport level once connected; the server/client role in postcard-rpc is the only asymmetry.
- **Topic granularity.** postcard-rpc's primitives map directly onto the traffic patterns: periodic sensor data (sim → FC, no response expected) uses Topics; commands that need an ack (FC → sim deployment, sim → FC arm trigger) use Endpoints. One multiplexed stream per peer pair; postcard-rpc handles the demultiplexing. No per-peripheral socket is needed.
- **`impl_software` gating.** The host-IPC simulated peripherals stay gated behind a feature flag, and PIL peripherals are a sibling implementation (`impl_pil` or similar) gated for embedded targets — *not* a transport-conditional fork of the same type.
- **Simulator runtime configuration.** The Simulator runs its own postcard-rpc server on `sim-gs.sock`; the GS backend connects to it as a client. Runtime config (change physics parameters, inject faults, retrigger ignition) and physics-status Topics flow directly between GS and Simulator without passing through FC. This keeps sim-config traffic off the FC entirely and avoids coupling simulator releases to GS releases. See Decision item 2 for the full three-socket triangle.

## Deferred work

The following questions do not block the binary split and are deferred to subsequent ADRs or implementation phases. Track progress in [`docs/TODO.md`](TODO.md).

- **FC / simulator reset semantics.** What happens when one peer restarts mid-scenario? Does the IPC reconnect transparently, or is the run aborted? Does the simulator rewind its physics state to match a fresh FC, or keep running and let the FC catch up? Specify before reset-spanning bugs become reproduce-only.
- **Incremental testing during the migration.** Decision #7 defers the eventual test rework, but the application is under active development *while* the binaries are being split. Pin the interim story: which tests stay on the in-process `Signal` harness, which migrate to IPC clients first, and how the migration avoids a regression gap. This gates how fast the rest of this ADR can be implemented.

## Consequences

- The FC library stops pulling simulator wiring into consumers. The `impl_software` flow becomes "postcard-rpc client calls over the interprocess socket"; no in-process simulator types compile in.
- PIL becomes expressible without inventing a second conceptual interface. It is the same peripheral traits and the same postcard-rpc client code, with the interprocess socket swapped for USB.
- One RPC framework (`postcard-rpc`) over two transport media (interprocess local socket on host, USB / radio for GS and PIL). The message vocabulary and the Topics/Endpoints API are single-sourced in `proto/`; only the `WireTx`/`WireRx` adapter differs per medium.
- The `InterprocessWireTx`/`WireRx` adapter is the only new transport component. It is a thin wrapper (~50 LOC) over the `tokio::io::split` halves of an `interprocess::local_socket::tokio::Stream`; everything else reuses existing postcard-rpc infrastructure.
- Existing FC unit tests that write to a static `Signal` will need rework. Deferred.
- The current [`impl_host`](../../code/flight-computer/Cargo.toml) feature and the in-process `Sim*` types become a stopgap to retire as the binaries split.
