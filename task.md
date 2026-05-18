lets work on milestone 2

Following the brainstorming checklist — I've explored the project context thoroughly. Here's what I've learned:
Current state: The code/simulator/ crate already has a physics engine (1D parabolic), hardcoded config, scripted events, and channel-based API — but it's a library-only crate designed for the old in-process architecture. M2.2 requires converting it into a standalone binary with:
1. postcard-rpc client connecting to fc-sim.sock (publish sensor Topics, receive deploy/LED Endpoints)
2. postcard-rpc server on sim-gs.sock (lifecycle, config-hash, manual triggers for GS)
3. Two-phase lifecycle (Setup → Runtime) with config loading from CLI
4. Config validation + hashing + GS handshake
5. ratatui TUI (physics state, lifecycle phase, LED state, manual triggers)
6. Structured tracing + panic hook (like flight-computer-host)
7. Physics tick loop driving postcard-rpc publishes
Blockers resolved: M1 is 100% complete, M2.1 (flight-computer-host) is built and running, and the connection diagram (three-socket triangle per ADR-001) is settled in the spec.
A few clarifying questions:
# Questions
The spec says the sim TUI is a separate panel in the simulator binary. Should we build the TUI in M2.2 or defer it to get the process boundary working first (just CLI-driven for now)?
Minimal TUI + defer full interactive TUI to M2.4, and making the physics engine 3D to M2.3 (with launch inclination, ...)
How should we handle the existing simulator library code? It uses channels (mpsc/broadcast) internally — the M2.2 binary needs to drive physics and publish via postcard-rpc instead.
i want you to take inspiration from it, ideally remove it, it was a preliminary solution
Config file format: what should the sim config file look like?
im not sure! what are the drawbacks?

Good answers. For config format — the key tradeoffs:
- TOML: Rust standard (Cargo uses it), supports comments, great serde support. Best for hand-edited configs. Slightly verbose.
- JSON: Universal, trivial serde, but no comments and stricter syntax for humans.
- YAML: Very readable, supports comments, but serde_yaml has had security/maintenance issues.
I'd recommend TOML — it's the Rust ecosystem standard for this kind of hand-edited config file, comments mean operators can document their scenarios inline.
Now let me lay out the approaches for the overall M2.2 implementation:
---
Approach A: Clean rewrite alongside (Recommended)
- New code/simulator/src/bin/simulator.rs binary entry point
- New modules for: fc_client (postcard-rpc connector to fc-sim.sock), gs_server (postcard-rpc server on sim-gs.sock), lifecycle (Setup→Runtime state machine), tui (minimal read-only), config (TOML-based)
- Port physics engine concepts (state, integration, force events) into a clean physics module — the old one was channel-based and needs restructuring
- Postcard-rpc adapter uses the existing InterprocessWireTx/Rx from proto/
# Questions
Does Approach A (clean rewrite alongside postcard-rpc) sound right? Or would you prefer one of the others?
Approach A: Clean rewrite (Recommended)
---
physics engine does not publish every tick, but the simulation runs every tick. the data publishing rate is different, usually slower. make this a const config and assert it is actually slower than. 
dont create a watch<ledState>, make an enum with SimActuatorType (led, deployment) and SimSensorType, and have two structs (SimActuator and SimSensor) that contain the data and its type!
wheres the events that are triggered with config aka ignition time, force, ...? who manages those?
---
Good catches, let me revise the architecture based on your feedback.
Revised architecture:
┌──────────────────────────────────────────────────────────────────┐
│                        simulator binary                          │
│                                                                  │
│   ┌───────────────────┐                                          │
│   │ ScriptedScenario  │──▶ mpsc::Sender<TriggerEvent>            │
│   │ Runner (from      │   (ignition @ t=5s, arm @ t=10s, ...)   │
│   │  config)          │                                          │
│   └───────────────────┘                                          │
│                                                                  │
│   ┌───────────────────┐      publish rate (e.g. 20ms)            │
│   │ Physics Engine    │──▶ broadcast::Sender<PhysicsState>        │
│   │ ticks at 1ms      │                                          │
│   │ receives triggers │     sim_time                              │
│   │ manages ForceEvts │──▶ watch::Sender<Duration>               │
│   └──────┬────────────┘                                          │
│          │ receives TriggerEvent                                  │
│          ▼                                                        │
│   ┌───────────────────┐   ┌───────────────────────────────────┐  │
│   │ FC Client         │   │ GS Server                         │  │
│   │ (postcard client  │   │ (postcard server on sim-gs.sock)  │  │
│   │  to fc-sim.sock)  │   │  - lifecycle Endpoints            │  │
│   │  - publishes      │   │  - manual trigger Endpoints       │  │
│   │    sensor Topics  │   │  - status Topics                  │  │
│   │    at publish_rate│   │  - config-hash handshake          │  │
│   │  - receives       │   └───────────────────────────────────┘  │
│   │    deploy/LED     │          │                               │
│   │    Endpoints      │          │ mpsc::Sender<TriggerEvent>     │
│   └──────┬────────────┘          ▼                               │
│          │ updates              triggers                          │
│          ▼                                                        │
│   ┌──────────────────────────────────────────┐                   │
│   │ Shared SimState                           │                   │
│   │  - SimSensor (type + sensor data)        │                   │
│   │  - SimActuator (type + status)           │                   │
│   │  - SimPhase (Setup/Running/Shutdown)     │                   │
│   └──────────────────────────────────────────┘                   │
│          ▲                                                        │
│   ┌──────┴────────────┐                                          │
│   │ Minimal TUI       │                                          │
│   │ (read-only, reads │                                          │
│   │  SimState + log)  │                                          │
│   └───────────────────┘                                          │
│                                                                  │
│   ┌───────────────────┐  ┌────────────────┐  ┌────────────────┐  │
│   │ Config (TOML)     │  │ Lifecycle FSM  │  │ Internal Log   │  │
│   │ physics + scenario│  │ Setup→Runtime  │  │ (NDJSON)       │  │
│   └───────────────────┘  └────────────────┘  └────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
Key changes from your feedback:
1. Two rates, explicitly distinguished. Config has physics_time_step (e.g. 1ms) and data_acquisition_interval (e.g. 20ms). Build-time assertion that acquisition >= step. The broadcast happens at the acquisition rate; the physics loop runs at the step rate internally.
2. SimState types instead of a catch-all LedState:
      enum SimSensorType { Altimeter, Gps, Imu }
   struct SimSensor { sensor_type: SimSensorType, last_data: PhysicsState }
   
   enum SimActuatorType { Deployment, PostcardLed, AltimeterLed, ... }
   struct SimActuator { actuator_type: SimActuatorType, status: ActuatorStatus }
   
3. Scripted Scenario Runner — dedicated async task that loads trigger schedule from config, monitors sim time via watch::Receiver<Duration>, and sends TriggerEvents (ignition, arm) into the same mpsc channel used by GS/TUI triggers. Force event management stays inside the physics engine.
---
tick rates (i.e. const values should come from config!) maybe we can load from config at compile time. then all const variables can also be stored there. wdyt?
what is sim_time used for? config (TOML) component should send its config through postcard to the GS (service), then GS compares the two. 
GS Server should not only act onto hte shared SimState (should SimState also contain the list of active events?), but also read / receive the SimSensor and SimActuator and publish it to GS! 
what is NDJSON?
---
Great points. Let me address each:
1. Tick rates from config — Yes, all rates (physics tick, data acquisition) go in the TOML file as runtime values, not hardcoded. The assertion acquisition >= step fires at load time. Optionally we can include_str! a default TOML at compile time so the binary works without a CLI arg for development.
2. sim_time — It's just part of PhysicsState. The ScriptedScenarioRunner reads PhysicsState.time from the broadcast to check trigger conditions. No separate channel needed.
3. Config → GS — Beyond just the hash handshake, the full config values should be published as a Topic to GS so GS can display them (and compare against its own copy if needed). Good call.
4. GS Server reads — Yes, GS server publishes status Topics containing: phase, physics state, active force event list, sensor values, actuator states, config hash. Full observability per §5.3.
5. NDJSON — Newline-Delimited JSON. Each line is {"sim_time":1.234,"altitude":45.6,...} — append-only, greppable, parseable with standard JSON tools. The internal sim log format.
---
On the SimState design — given your feedback, I'm thinking:
Central SimState via individual watch channels (no Arc<RwLock<> needed):
Physics task owns writes to:
  ─ watch::Sender<PhysicsState>      (at data_acquisition rate)
  ─ watch::Sender<Vec<ActiveEvent>>   (force event summaries)
  ─ watch::Sender<SharedSensors>      (converted sensor data)
FC client task owns writes to:
  ─ watch::Sender<SharedActuators>    (LED status, deployment status)
Lifecycle owns writes to:
  ─ watch::Sender<SimPhase>           (Setup / Running / Shutdown)
All others (TUI, GS server, scripted runner) read via watch::Receiver
Where:
struct SharedSensors {
    altimeter: SimSensor<SimSensorType::Altimeter>,  // last published data
    gps: SimSensor<SimSensorType::Gps>,
    imu: SimSensor<SimSensorType::Imu>,
}
struct SharedActuators {
    deployment: SimActuator<SimActuatorType::Deployment>,
    postcard_led: SimActuator<SimActuatorType::PostcardLed>,
    altimeter_led: SimActuator<SimActuatorType::AltimeterLed>,
    // ... all 8 LEDs
}
No single monolithic SimState struct — just parallel watches that component tasks compose naturally. This way:
- Physics doesn't need to know about actuators
- FC client doesn't need to know about physics engine internals
- Each task owns exactly what it produces
Does this design approach for shared state and the watch/task ownership model feel right?
# Questions
Does this shared-state design (parallel watch channels, no monolithic SimState, task-owned writes) feel right?
Almost — one more tweak
1. we can skip the CLI arg! 
btw we should have tracing_append just like in @code\flight-computer-host\src\logging.rs! 
on the diagram you've been writing, can you represent the communication to other binaries (GS and FC), as arrows that point outside the box. this way i can better visualize what are interactions between components of the simulator, and what are interactions between external components
we actually dont want a sharedSensors and sharedActuators, the Physics state is all that matters, then we have a from physics state for the sensor readings, thats all.
i was thinking more: without SharedSensors, we could have: SimSensorData, which are not converted sensor data, its an enum: enum { Imu(ImuData), GPS (GpsData), ... }
and SharedActuators works the same way! if necessary this abstracts the actual sensor data type being worked on, be it reading physics state and translating it to one of these, dealing with the actuator or sensor data in other components inside the sim. wdyt?
---
Good feedback. Let me incorporate everything:
Config loading: Skip CLI arg — load from a known default path (e.g. ./sim_config.toml). Keeps things simple for M2.2.
Logging: Mirror flight-computer-host/src/logging.rs — per-level JSON files + stdout + panic hook. Consistency across binaries.
Communication diagram — let me redraw with clear external vs internal boundaries.
Sensor/actuator data: You're right — PhysicsState is the source of truth for sensors. Components that need specific sensor data convert via From<PhysicsState>. For actuators, the FC client receives postcard-rpc Topics that need to be stored for TUI/GS to read. I'll use:
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
Stored in a watch::Sender<Vec<SimActuatorData>> (FC client writes, TUI/GS server reads).
---
Revised diagram — external arrows go outside the box:
                      ┌──────────────────────┐
                      │   flight-computer-   │
                      │   host               │
                      │   (postcard-rpc      │
                      │    SERVER on          │
                      │    fc-sim.sock)       │
                      └─────────┬────────────┘
                                │ fc-sim.sock
                                │ (connects)
                                │
   ┌────────────────────────────┴───────────────────────────┐
   │                   simulator binary                     │
   │                                                       │
   │  ╔═══════════════════════════════════════════════════╗ │
   │  ║  FC Client (postcard-rpc CLIENT to fc-sim.sock)  ║ │
   │  ║  publishes: SimAltimeterTopic, SimGpsTopic,      ║ │
   │  ║             SimImuTopic, SimArmTopic              ║ │
   │  ║  receives:  SimDeploymentTopic, Sim*LedTopic     ║ │
   │  ╚═══════════════════════╤═══════════════════════════╝ │
   │                          │                              │
   │                          │ publishes at                 │
   │                          │ data_acquisition_rate        │
   │                          ▼                              │
   │  ┌──────────────────────────────────────────────────┐  │
   │  │  Physics Engine                                  │  │
   │  │  - ticks at physics_time_step (1ms)              │  │
   │  │  - accumulates time, integrates forces           │  │
   │  │  - broadcasts PhysicsState at pub rate            │  │
   │  │  - manages ActiveEvent list (force events)       │  │
   │  │  - receives TriggerEvents via mpsc               │  │
   │  └──────────┬──────────────┬───────────────────────┘  │
   │             │              │                           │
   │        watch::Sender   watch::Sender                   │
   │        <PhysicsState>  <Vec<ActiveEvent>>              │
   │             │              │                           │
   │             ▼              ▼                           │
   │  ╔═══════════════════════════════════════════════════╗ │
   │  ║  GS Server  (postcard-rpc SERVER on sim-gs.sock) ║ │
   │  ║  serves:    Start, Restart, Shutdown Endpoints   ║ │
   │  ║             ActivateArm, MotorIgnition, Deploy   ║ │
   │  ║  publishes: SimPhase, PhysicsState,              ║ │
   │  ║             ActiveEvents, ActuatorState,         ║ │
   │  ║             ConfigHash Topics                    ║ │
   │  ╚═══════════════════════╤═══════════════════════════╝ │
   │                          │                              │
   │  ┌──────────────────┐   │   ┌──────────────────────┐   │
   │  │ ScriptedScenario │   │   │ Minimal TUI          │   │
   │  │ Runner           │───┼──▶│ (reads: PhysicsState, │   │
   │  │ (reads sim_time, │   │   │  active events,      │   │
   │  │  fires triggers  │   │   │  actuator states,    │   │
   │  │  at configured   │   │   │  phase, config hash) │   │
   │  │  times)          │   │   └──────────────────────┘   │
   │  └──────────────────┘   │                              │
   │                          │                              │
   │  ┌──────────────────┐   │   ┌──────────────────────┐   │
   │  │ Actuator State   │◀──┘   │ Config               │   │
   │  │ (watch channel   │       │ (TOML from           │   │
   │  │  FC client fills,│       │  ./sim_config.toml)  │   │
   │  │  TUI/GS reads)   │       │                      │   │
   │  └──────────────────┘       │  validation + hash   │   │
   │                             └──────────────────────┘   │
   └────────────────────────────┬───────────────────────────┘
                                │ sim-gs.sock
                                │ (listens)
                                │
                      ┌─────────┴────────────┐
                      │   ground-station-    │
                      │   backend            │
                      │   (postcard-rpc      │
                      │    CLIENT on          │
                      │    sim-gs.sock)       │
                      └──────────────────────┘
Trigger flow:
[GS Endpoint]  ──▶  mpsc::Sender<TriggerEvent>  ──▶  Physics Engine
[Scripted]     ──▶  mpsc::Sender<TriggerEvent>  ──▶  Physics Engine
where:
enum TriggerEvent {
    Ignition(Duration start_time),
    Arm,
    Deploy,
}
Actuator update flow:
FC Client ←── SimDeploymentTopic / Sim*LedTopic (from FC)
FC Client ──▶ watch::Sender<Vec<SimActuatorData>> ──▶ TUI, GS Server
---
Same purpose as flight-computer-host — per spec §10.3, every host process must install a panic hook. For the simulator specifically:
1. Trace capture — If the physics loop or FC client panics, the reason is written to the structured JSON logs before exit. Without it, you just get a stderr message with no correlation to the sim's trace context.
2. Consistency — All three host processes (FC-host, simulator, GS backend) follow the same pattern. Makes debugging deterministic — always check the log dir.
---
isnt there a hash type in std rust? or is it just a trait? 
i dont like functions called fn_* make them a descriptive name!
remember you can use tokio to spawn tasks instead of select!
remember what we talked about shutdown and restart resilience on simulator, gs, and fc
---
Good points. Let me address each:
Hash: [u8; 32] is the right type — std::hash::Hash is a trait (not a type). We'll use the sha2 crate to compute SHA-256 of the serialized config.
Naming: Agreed — no fn_* prefixes. Descriptive names instead.
Spawning: You're right — use tokio::spawn for each domain, not a single select!. The lifecycle loop orchestrates which tasks live at which phase.
Shutdown/restart resilience recap:
- FC crashes → FC client detects ConnectionClosed → signals lifecycle to shutdown (run-lifecycle peer)
- GS disconnects → GS server logs it, keeps listening (GS is observational, sim keeps running)
- Restart → cancel run-scoped tasks, reset physics to t=0, re-enter Setup, wait for Start
- Shutdown → cancel all tasks, flush logs, exit 0
---
Revised architecture with spawned tasks:
┌─────────────────┐
│   main()        │  Lifecycle loop (select! on lifecycle_rx)
│                 │
│  spawn:         │   GS server task ─── runs forever
│  │              │   TUI task       ─── runs forever
│  │              │
│  │  on Start:   │   spawn physics_loop_task ─── cancellable
│  │  │           │   spawn fc_client_task    ─── cancellable
│  │  │           │   spawn scripted_runner   ─── cancellable
│  │  │           │
│  │  on Restart: │   cancel run tasks
│  │              │   reset state → Setup phase
│  │              │
│  │  on Shutdown:│   cancel all → exit
│  └──────────────┘
└─────────────────┘
---
One question I want to resolve before writing the spec doc: should the FC client reconnect on restart (when the sim re-enters Setup after a Restart command), or should it maintain the connection across restarts? The spec says FC and Sim are run-lifecycle peers — a restart is a clean re-initialization, so reconnecting feels correct.
# Questions
On Restart — should the FC client disconnect and reconnect (clean socket), or keep the connection alive and just reset physics?
this is not an issue! if fc and sim pipe is broken, panic sim! even during execution this should trigger a panic / restart of FC but that will have to be solved in M2.4; fc-gs and sim-gs pipe is not critical, dont pause sim, reconnect
---
actually, i want to start small... lets do scripted scenario from compile time config (on the code - config struct), minimal tui, fc client and physics engine! anything else is postponed to a later date. all gs relevant stuff gets postponed to M3.2+, and the rest to M2.4+! 
also, ive been postponing a lot of stuff during this conversation. i want you to edit @docs\ROADMAP.md to reflec this, and if you need, update other relevant documentation;
afterwards, well analyze this simpler approach/ MVP
---
M2.2 — Simulator binary MVP
Four modules, ~450 LOC total:
Module	File	Role
Physics	physics/engine.rs, state.rs, events.rs	1D parabolic integration, force events (thrust, gravity, drag), PhysicsState
FC client	fc_client.rs	Connect to fc-sim.sock, publish sensor Topics from PhysicsState, receive deploy/LED
Scripted	scripted.rs	Watch PhysicsState.time, fire Ignition/Arm at configured delays
TUI	tui.rs	ratatui read-only: altitude, velocity, acceleration, sim time, LED states, events
Config	config.rs	pub const DEFAULT struct — all physics + timing + socket path
Logging	logging.rs	tracing_appender per-level JSON + stdout + panic hook (copy from fc-host)
Types	types.rs	TriggerEvent, SimActuatorData, ActiveEventSummary
Main	main.rs	Wire 4 channels, spawn FC client + scripted + TUI, run physics loop
The flow:
Binary start
  → init tracing
  → connect to FC-host on fc-sim.sock (blocking)
  → spawn FC client task (publishes sensor Topics at data_acquisition rate)
  → spawn scripted runner task (fire triggers on schedule)
  → spawn minimal TUI task
  → run physics loop:
       every 1ms: step physics, check trigger_rx
       every 20ms: broadcast PhysicsState → FC client + TUI
Channels: watch<PhysicsState>, watch<Vec<ActiveEvent>>, watch<Vec<SimActuatorData>>, mpsc<TriggerEvent>
No: GS interaction, lifecycle management, config file, config hashing, interactive TUI.
---
run physics loop -> spawn physics loop;
also, support graceful shutdown! (through ctrl+c)
---
Rust doesn't have a built-in like Go's context.Context, but tokio_util::sync::CancellationToken provides the same pattern:
use tokio_util::sync::CancellationToken;
let cancel = CancellationToken::new();
// Spawn tasks with cloned tokens
tokio::spawn({
    let cancel = cancel.clone();
    async move {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,  // same as <-ctx.Done()
                _ = ticker.tick() => step(),
            }
        }
    }
});
// Trigger all tasks to stop
cancel.cancel();
// Wait for them to finish
join_handle.await.unwrap();
It's functionally equivalent to Go's ctx.Done() — a broadcast signal that all tasks can select! on. The main difference: no defer, so you put cleanup before break manually.
Three patterns exist:
Pattern	How it works
CancellationToken (tokio_util)	Broadcast signal, tasks check cooperatively
Drop JoinHandle	Tokio drops the task at next .await
Channel close	rx.recv() returns None when all senders drop
For M2.2, CancellationToken is the right choice — each task checks it at the top of its loop, cleans up (TUI restores terminal, FC client flushes), then exits.
---
Where should the M2.2 design spec live?
code/simulator/SPEC.md (Recommended)
update the spec iteratively through every message! 
also add a README, which presents a brief overview. In contrast, spec.md should cover the detailed design.
---
actually, this simulator should be structured as a lib + bin. there will be 2 binaries: simulator for host and for pil! the only difference in these is initializing the transport layer for postcard (pil uses usb for sim and gs)! everything else should be compatible! to implement this, we should use @code\ground-station-backend\src\bin\serial/ to copy from any relevant code. 
additionally, in @code\flight-computer\src\tasks\simulation.rs postcard_sim_task should panic if errored - fc + sim desync! 
---
What changed from the original M2.2 plan
Original scope	New scope (MVP)	Deferred to
Two-phase lifecycle (Setup/Runtime)	Run on launch, Ctrl-C to exit	M2.4
Config from CLI + TOML file	Compile-time pub const SIM_CONFIG	M2.4
Config hashing + GS handshake	—	M3.3
sim-gs.sock server	—	M3.3
GS lifecycle/trigger Endpoints	—	M3.3
Interactive TUI (keyboard)	Read-only display only	M2.4
3D physics	1D parabolic trajectory	M2.3
Architecture decisions made
1. Lib + bin structure — simulator is a library crate containing shared logic (physics, types, logging, physics runner), with two binary entry points:
   - host — interprocess socket transport (fc-sim.sock)
   - pil — USB nusb transport (HostClient::try_new_raw_nusb)
   
2. Transport abstraction — The library exposes a generic run_simulator(client: impl SimTransport, ...) or accepts a HostClient<WireError> that both binaries construct differently.
3. postcard_sim_task panic — In flight-computer/src/tasks/simulation.rs, the postcard server task (for sim peripheral surface) should panic on error instead of logging and retrying. FC ↔ Sim desync is a critical failure.
4. Graceful shutdown — CancellationToken propagated to all spawned tasks (physics loop, FC client, TUI, scripted runner). Ctrl-C triggers orderly teardown.
Milestone structure (final)
M#	Task	Status
M2.1	flight-computer-host binary	Done
M2.2	Simulator binary MVP (this task)	Ready
M2.3	3D physics enhancement	Deferred
M2.4	Config file, lifecycle, interactive TUI	Deferred
M3.1	GS backend (FC-facing)	Blocked
M3.2	GS frontend TUI	Blocked
M3.3	Simulator-GS integration	Blocked
