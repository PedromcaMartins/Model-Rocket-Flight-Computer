# ADR-001: Split host build into per-role binaries with postcard-rpc as unified RPC layer

- **Status:** Accepted
- **Date:** 2026-05-05
- **Revised:** 2026-05-07 — added Option 4 (hub topology) and expanded each option to a full sub-chapter.

> The contract this decision produced — three-socket triangle, server/client roles, transport adapter, startup order — lives in [`../software/spec.md` §8](../software/spec.md#8-host-ipc). This ADR captures only *why* this approach was chosen over the alternatives.

---

## Context

The HOST deployment mode was a single binary that wired the flight-computer (FC) library, the simulator, and the ground-station backend together with in-process Embassy channels and direct library calls. This caused three problems:

- The simulator and the FC could not be run independently. `cargo run -p simulator` and `cargo run -p flight-computer` existed as crates but could not talk to each other.
- PIL (FC firmware on the prod board, simulator on host) had no implementation. It needed the same simulated-peripheral surface the host build uses, but routed over USB to the MCU.
- The FC library had to drag the simulator's wiring along even when the consumer didn't want it.

The simulator drives more than just sensors. The FC's peripheral surface is multi-trait: [`Sensor`](../../code/flight-computer/src/interfaces/sensor.rs), [`ArmingSystem`](../../code/flight-computer/src/interfaces/arming_system.rs), [`DeploymentSystem`](../../code/flight-computer/src/interfaces/deployment_system.rs), [`Led`](../../code/flight-computer/src/interfaces/led.rs). The [`FileSystem`](../../code/flight-computer/src/interfaces/filesystem.rs) is the only peripheral with a real I/O even in HOST and PIL — every other peripheral has a simulated implementation. Any IPC scheme had to carry all of them, in both directions.

The key structural asymmetry that any topology must respect:

- **FC ↔ Sim** is a *tight, bidirectional, high-cadence* link. The simulator drives sensor Topics into FC at every physics tick (~10 Hz); FC drives deployment and LED Endpoints back into the simulator. Both sides are run-lifecycle peers — if one dies, the run is over.
- **FC ↔ GS** and **Sim ↔ GS** are *observational and control links*, not peripheral links. GS reads telemetry, issues operator commands, manages scenario config, and surfaces status to the operator. It is not in the physics tick path.

---

## Options considered

### Option 1 — Stay in-process (status quo)

**Description.** The single-binary model: FC library, simulator, and GS backend all live in one process, wired together via in-process Embassy channels and direct function calls. No sockets, no IPC. `cargo run -p simulator` and `cargo run -p flight-computer` exist as crates but cannot communicate without the monolith entry point.

**Pros:**
- Zero IPC overhead. Sensor ticks, peripheral calls, and telemetry are all blocking-fast in-process.
- Simplest possible deployment: one process to start, one to kill.
- Unit tests that write to a static `Signal` continue to work unchanged.
- No socket startup ordering, no reconnect semantics, no transport adapters to write.

**Cons:**
- **Does not solve PIL.** PIL requires the FC firmware to run on the production MCU while the simulator runs on host. There is no in-process path for this; a second interface would need to be invented anyway.
- **Binaries cannot be run independently.** Neither `cargo run -p simulator` nor the FC library is independently exercisable without the whole monolith wiring.
- **FC library drags simulator types into every consumer.** Code that only wants FC must still compile against simulator wiring.
- **FC and GS concerns bleed together.** Sim control, telemetry, and peripheral data all share the same in-process call graph; there are no explicit boundaries to enforce separation.
- **No path to production topology.** HW mode has no simulator at all. Modeling that is artificial if everything is in-process.

**Verdict:** Cheapest to maintain in the short term, but blocks PIL and violates the "one FC library, three deployment targets" goal from day one.

---

### Option 2 — Single transport for everything (one wire format, host and USB)

**Description.** Extract the binaries but unify transport: one wire protocol runs over both host-local IPC and USB. Either the host case adapts to the USB framing (e.g. a virtual serial port), or the USB case adapts to a host-local primitive (e.g. a Unix socket tunnelled over USB). A single `proto/` vocabulary; a single codec path shared by all links.

**Pros:**
- Uniform stack: one codec crate, one set of tests, one mental model for all links.
- PIL is expressible without a conceptually different interface.
- Simulator and FC can each run as standalone binaries.

**Cons:**
- **Transport impedance mismatch.** Host-local IPC wants low-latency, kernel-buffered, byte-stream sockets. USB wants framed serial with line discipline, COBS encoding, and a flow-control protocol. Making one codec serve both without losing properties (latency on host, framing correctness on USB) forces either artificial overhead on the host path or lossy adaptation on the USB path.
- **Multiplexing is non-trivial on USB.** In PIL, FC telemetry to GS and simulator peripheral data to FC must share one USB wire. A shared codec needs explicit mux/demux logic; this is the one new component Option 3 avoids by keeping the transport adapter thin.
- **Simulation-only traffic enters the USB vocabulary.** If everything is one wire format, deployment LED commands and sensor Topics must be expressible over USB even when the HW binary has no simulator. That either pollutes the `proto/` with simulator-only types or adds a feature-gated vocabulary split that erodes the "single wire vocabulary" goal.
- **Effectively reimplements postcard-rpc's transport layer.** `postcard-rpc` already provides the Topic / Endpoint model and handles the message framing; adapting it to two physical media is precisely what its transport adapter interface is designed for. Bypassing it loses the design without gaining anything.

**Verdict:** Uniform in principle, but the transport mismatch requires more code than Option 3 and degrades at least one deployment medium.

---

### Option 3 — Three-socket triangle: FC ↔ Sim direct, both ↔ GS (adopted)

**Description.** Split HOST into four binaries (FC, simulator, GS backend, GS frontend). Three sockets:

```
   FC ──── fc-sim.sock ────► Sim      (peripheral surface: sensors, arm, deploy, LED)
   FC ──── fc-gs.sock  ────► GS-BE    (telemetry, commands)
  Sim ──── sim-gs.sock ────► GS-BE    (lifecycle, config-hash, triggers, status)
```

`postcard-rpc` runs over all three. The transport adapter (`InterprocessWireTx`/`Rx`) is the only new component. On USB (PIL) the same postcard-rpc vocabulary runs over a framed serial link; only the `WireTx`/`WireRx` adapter swaps.

**Pros:**
- **FC ↔ Sim link is direct and dedicated.** No third process is in the tick path. Sensor Topics flow at physics cadence without an intermediary that could buffer, slow down, or fail independently.
- **PIL maps cleanly.** `impl_host` → `impl_sim` by swapping the interprocess socket adapter for a USB adapter. The same peripheral traits, the same postcard-rpc server code, the same `proto/` vocabulary.
- **GS is genuinely observational.** It is not a hub; it cannot become a bottleneck or single point of failure for the physics loop. FC and Sim continue running if GS crashes (spec §10 crash policy).
- **Sim-control traffic stays off the FC entirely.** `sim-gs.sock` carries lifecycle, config-hash handshake, manual triggers, and physics status. FC has zero awareness of operator-only concerns.
- **Single wire vocabulary.** `proto/` Topics and Endpoints are shared across all three links; only the transport adapter differs.
- **Thin transport component.** `InterprocessWireTx`/`WireRx` is ~50 LOC over `tokio::io::split` halves of an `interprocess` stream.
- **Crash semantics are clean.** FC ↔ Sim are peers (either dying ends the run); GS is optional (its absence is degraded but not fatal). The asymmetry is architecturally visible in the topology.

**Cons:**
- **Three sockets to manage.** Startup ordering matters (`fc-sim.sock` must exist before FC connects; `sim-gs.sock` and `fc-gs.sock` must exist before GS connects). `xtask` handles this, but it is more orchestration than a single-socket or hub design.
- **GS has no direct Sim ↔ FC visibility.** GS cannot observe or intercept the peripheral surface traffic. If GS needs to log raw sensor publications for replay, it must get them indirectly via telemetry Topics from FC, not by intercepting `fc-sim.sock`.
- **Unit tests that write to a static `Signal` need rework.** Deferred; tracked in [`../TODO.md`](../TODO.md). The migration does not change the peripheral trait contracts, only the implementation layer.

**Verdict:** Adopted. The direct FC ↔ Sim link preserves the tight peripheral coupling without intermediary risk; the two GS sockets are both thin and well-scoped. PIL re-use requires no second server or second link.

---

### Option 4 — Hub topology: FC ↔ GS-BE ↔ Sim, GS-BE ↔ GS-FE

**Description.** Remove the direct FC ↔ Sim socket. Route all traffic through GS-BE as a central hub. Two sockets:

```
   FC  ──── fc-gs.sock  ────► GS-BE ◄──── sim-gs.sock ──── Sim
                              GS-BE ◄──── gs-fe.sock  ──── GS-FE
```

GS-BE becomes the broker for both telemetry/commands (FC ↔ GS-BE) and the peripheral surface (Sim ↔ GS-BE ↔ FC). It receives sensor Topics from Sim, forwards them to FC, and forwards FC's deployment/LED calls back to Sim.

**Pros:**
- **Single connectivity point.** GS-BE is the only process every other process connects to. No explicit `fc-sim.sock` startup ordering; both FC and Sim independently establish their connection to GS-BE.
- **GS has full visibility into peripheral traffic.** Because all FC ↔ Sim data flows through GS-BE, GS can log raw sensor publications, replay them, and correlate peripheral state with telemetry in a single place without aggregating from multiple sources.
- **Symmetric connection model for FC and Sim.** Both FC and Sim are pure clients of GS-BE. No server/client role asymmetry between FC and Sim; both point at the same hub.
- **Sim and FC are fully decoupled from each other.** Neither binary has any knowledge of the other's socket address, identity, or lifecycle; both interact only with GS-BE.
- **Fewer socket files.** Two sockets instead of three reduces filesystem artefacts and the manifest of socket paths that `xtask` and the operator must track.

**Cons:**
- **GS-BE is now in the physics tick path.** Every sensor Topic the simulator produces must pass through GS-BE before FC can read it. Every deployment or LED call FC makes must pass through GS-BE before Sim receives it. GS-BE becomes a required intermediary on the hot path. If GS-BE crashes, both FC and Sim lose their peer — the run is over. This inverts the crash semantics in spec §10: GS is no longer "optional and observational"; it is a run-lifecycle requirement.
- **GS-BE must implement peripheral forwarding logic.** Today GS-BE only speaks the telemetry and command vocabulary. Under this option it must also implement correct forwarding of the peripheral surface: Topic fan-out timing, Endpoint round-trip latency, backpressure handling between the Sim→GS-BE leg and the GS-BE→FC leg. This is non-trivial; it introduces a new failure mode (GS-BE drops or re-orders a sensor tick) that does not exist when FC and Sim communicate directly.
- **Additional latency on every sensor tick.** In the triangle topology, a sensor Topic travels one hop (Sim → FC). In the hub topology it travels two (Sim → GS-BE → FC). On host, this is probably unobservable in untimed testing, but it adds a gratuitous serialisation point.
- **PIL breaks the topology.** In PIL, FC runs on the production MCU and communicates over USB. Under the hub model, the USB link connects FC to GS-BE (as it does today in HW mode), but Sim must also communicate with GS-BE over a separate host-local link, and GS-BE must now forward peripheral surface traffic between them. The USB multiplexing problem (two logical channels — peripheral surface and telemetry — over one physical wire) is solved in Option 3 by having the simulator connect directly to the MCU over USB; in the hub model, GS-BE must demux two streams coming from different processes and re-mux them onto the USB wire. This is significantly more complex than `impl_sim`.
- **`proto/` vocabulary must expose the peripheral surface as a GS-BE-facing API.** Today the peripheral Topics and Endpoints are strictly a Sim ↔ FC contract; GS-BE has no involvement. Under the hub topology, GS-BE must parse, route, and forward these messages. The clean boundary between "peripheral surface" and "telemetry/control" disappears.
- **Violates the "sim-control stays off the FC" invariant.** Under this topology, GS-BE routes both peripheral surface traffic and sim-control traffic. While GS-BE can logically separate them internally, the FC's peripheral implementation now connects to the same process that handles lifecycle, config-hash handshake, and operator manual triggers. The separation that Option 3 enforces at the socket level must instead be enforced at the application level inside GS-BE — a weaker guarantee.

**Verdict:** The hub model trades a direct Sim ↔ FC socket for centralised observability and a simpler connection model, but at the cost of making GS-BE load-bearing in the physics tick path. This directly contradicts the spec's crash policy (§10), which is predicated on GS being optional, and it complicates PIL by requiring GS-BE to mediate between two host-side processes and the USB wire. The observability gain (GS can see raw sensor publications) can be recovered in Option 3 by having Sim also publish a mirrored sensor-log Topic on `sim-gs.sock`; no hub is required for that.

---

## Decision

Adopt **Option 3** (three-socket triangle).

The full topology — three sockets, server/client roles, startup order, transport adapter, reconnect semantics — is specified in [`../software/spec.md` §8](../software/spec.md#8-host-ipc). Other concerns the implementation depends on are split out:

- Peripheral-trait contract carried over `fc-sim.sock` → [`../software/spec.md` §5.1](../software/spec.md#51-fc--simulator-peripheral-surface).
- Simulator lifecycle, config ownership (GS-authoritative), and event model carried over `sim-gs.sock` → [`../software/spec.md` §7](../software/spec.md#7-simulator--lifecycle-config-events).
- Crash and disconnect behaviour → [`../software/spec.md` §10](../software/spec.md#10-crash--disconnect-policy).

### Why Option 3 over Option 4 specifically

The decisive factors, in order:

1. **Crash semantics.** The spec's crash policy (§10) is built on a specific asymmetry: FC and Sim are run-lifecycle peers; GS is observational. Option 4 promotes GS-BE to a run-lifecycle requirement. Every sensor tick and every peripheral call flows through it; if it goes down, the run ends. This is not a minor operational inconvenience — it is an architectural regression that removes the ability to keep the run alive while the operator restarts the UI.

2. **PIL feasibility.** Option 3 maps PIL cleanly: swap the interprocess socket adapter for a USB adapter, same postcard-rpc server, same `proto/` vocabulary. Option 4 requires GS-BE to demux two logical channels (peripheral surface traffic from Sim, telemetry from FC) arriving over different transports (host socket vs USB serial) and re-mux them correctly. This is a substantially larger implementation surface for PIL than Option 3 requires.

3. **Peripheral surface ownership.** The FC ↔ Sim peripheral boundary (`Sensor`, `ArmingSystem`, `DeploymentSystem`, `Led`) is a contract between the FC library's trait definitions and the simulator's implementations of those traits. Routing it through GS-BE forces GS-BE to understand and correctly forward a contract it has no architectural stake in. Option 3 keeps the peripheral contract where it belongs: between the two processes that define and implement it.

4. **The observability gap is small and closeable.** Option 4's main structural advantage is that GS-BE can see raw sensor publications. Option 3 recovers this without a hub: the simulator can emit a mirrored sensor-log Topic on `sim-gs.sock` if post-run sensor replay is required. This adds one Topic to the sim→GS vocabulary; it does not require restructuring the topology.

---

## Consequences

- The FC library stops pulling simulator wiring into consumers. `impl_host` becomes "postcard-rpc client over the interprocess socket"; no in-process simulator types compile in.
- PIL becomes expressible without inventing a second conceptual interface. Same peripheral traits, same postcard-rpc client code; the interprocess socket is swapped for USB.
- One RPC framework over two transport media. The message vocabulary and the Topics/Endpoints API are single-sourced in `proto/`; only the `WireTx`/`WireRx` adapter differs per medium.
- The `InterprocessWireTx`/`WireRx` adapter is the only new transport component (~50 LOC over `tokio::io::split` halves of an `interprocess::local_socket::tokio::Stream`).
- Existing FC unit tests that write to a static `Signal` will need rework. Deferred — tracked in [`../TODO.md`](../TODO.md).
- Sim-config and sim-control traffic stays off the FC entirely (lives on `sim-gs.sock`). FC has no awareness of operator-only concerns.
- Test ergonomics deferred. Reworking unit tests that today write to a static `Signal` is a future problem; it does not block the binary split.

---

## Deferred work

The following do not block the binary split. Tracked in [`../TODO.md`](../TODO.md):

- **FC / simulator reset semantics.** What happens when one peer restarts mid-scenario? The connection-level answer (single socket per pair, peer-crash ends the run) is in [`../software/spec.md` §8](../software/spec.md#8-host-ipc); the application-level question (does the simulator rewind physics state to match a fresh FC, or keep running?) is unresolved.
- **Incremental testing during the migration.** Pin which tests stay on the in-process `Signal` harness, which migrate to IPC clients first, and how the migration avoids a regression gap.

---

## See also

- [`../software/spec.md` §8](../software/spec.md#8-host-ipc) — the IPC topology and transport contract this decision produced.
- [`../ROADMAP.md`](../ROADMAP.md) — implementation milestones for the binary split.
