# Requirements

Every requirement has a **Rationale** (why it exists) and a **Verification**
(how it is confirmed met). See `docs/how-we-work.md §Traceability policy`.

---

## Development Methodology

### [DEV-1] Open Rocket / RocketPy simulations
Open Rocket / RocketPy simulations must be performed to evaluate and define
further Rocket System requirements.

- **Rationale:** Simulation is the cheapest way to discover sizing, stability,
  and performance problems before committing to hardware. Drives motor selection,
  airframe geometry, and recovery requirements.
- **Verification:** Simulation results (plots, export files) are reviewed and
  stored in `open rocket/` or referenced from `docs/`. Key outputs (stability
  margin, apogee altitude, max velocity) are documented.

### [DEV-2] Full CAD design
The rocket must be fully designed in CAD software.

- **Rationale:** CAD enables precise dimensioning, fit checks between
  subsystems (avionics bay ↔ airframe), mass budgeting, and printable/ machinable
  part generation. Without CAD, integration failures are discovered at assembly.
- **Verification:** All structural parts have corresponding CAD files in
  `structure/`. An assembly drawing shows every part in its assigned position.

### [DEV-3] Recovery system testing
Recovery systems must be tested: system activation testing and load testing.

- **Rationale:** Deployment failure is a total-loss event. Activation testing
  confirms the mechanism fires reliably; load testing confirms the parachute
  and attachment points survive the opening shock.
- **Verification:** Test log in `docs/` or linked from `docs/TODO.md` records
  activation test results (pass/fail, date, configuration) and load test data
  (force applied, duration, outcome).

### [DEV-4] Avionics system testing
Avionics systems must be tested.

- **Rationale:** Electronics failures (sensor drop, regulator brownout, radio
  link loss) are the most common failure mode in model rocketry. Subsystem-level
  testing catches them before integration.
- **Verification:** Test procedures in `docs/` or `hardware/` document
  pass/fail for each avionics subsystem (power rail, each sensor, radio,
  deployment pyro channel).

### [DEV-5] Propulsion / structural pre-launch testing
Propulsion or Structural systems don't have to be tested before launch.

- **Rationale:** Commercial motors are certified by the manufacturer; structural
  margins from CAD/FEA are sufficient at this scale. Testing would require
  specialized equipment (test stand, pull-tester) disproportionate to the risk.
- **Verification:** N/A — explicitly not tested. Motor selection relies on
  manufacturer data; structural margins are documented in CAD/FEA notes.

---

## Rocket system

### [ROCKET-1] Motor class limit
The rocket must use at max, an E-class motor and must be commercially available.

- **Rationale:** Legal/regulatory limit for low-power rocketry in the target
  operating region. E-class also bounds airframe sizing, mass budget, and
  recovery requirements. Doesn't require special motor license (L1, ...).
- **Verification:** Motor specification in the OpenRocket file or flight log
  shows total impulse ≤ 80 N·s (E-class limit). Purchase receipt or vendor
  listing confirms commercial availability.

### [ROCKET-2] Passive stabilization
The rocket must have passive stabilization.

- **Rationale:** Passive fins eliminate the complexity, mass, and failure modes
  of active thrust-vector or canard control. Sufficient for the target flight
  envelope.
- **Verification:** OpenRocket simulation shows static margin ≥ 1.0 calibre
  (or the project-defined minimum) across the full Mach range. CAD model
  includes fin geometry.

### [ROCKET-3] Recovery system
The rocket must have a recovery system.

- **Rationale:** Required by safety code (avoid ballistic return). Also
  protects onboard avionics for post-flight data recovery.
- **Verification:** Deployment mechanism (ejectable nose cone / baffle /
  drogue) is present in the CAD model and OpenRocket file. Activation test
  per DEV-3 passes.

### [ROCKET-4] Avionics for flight data logging
The rocket must have an avionics system to log flight data.

- **Rationale:** Post-flight analysis requires recorded sensor data. Without
  onboard logging, the entire flight is a black box.
- **Verification:** Flight computer includes non-volatile storage (SD / flash).
  A post-flight download procedure exists and produces readable output.

### [ROCKET-5] Reusable parts
Parts should be made reusable.

- **Rationale:** Reduces per-launch cost and enables iterative improvement.
  Printed/machined parts that survive one flight should fly again.
- **Verification:** Post-flight inspection checklist confirms which parts are
  reusable. Recovery system design minimises single-use components (e.g.
  screw-together couplers instead of shear-pin single-use).

### [ROCKET-6] Strong but affordable parts
Parts should be strong, but affordable.

- **Rationale:** Over-engineering drives cost without measurable benefit at
  this scale. Target "good enough" with a known margin.
- **Verification:** Material choice and wall thickness are documented in CAD
  with a brief reasoning note (e.g. "PLA, 3 walls, 15% infill — sufficient
  for 50 N·s impulse"). No requirement to test to destruction.

### [ROCKET-7] 3D printed parts
Parts should be 3D printed, if possible.

- **Rationale:** FDM printing is the fastest iteration cycle for custom
  geometry. Avoids CNC lead time and minimum-order quantities.
- **Verification:** Part files in `structure/` are in STL/3MF/STEP format
  suitable for slicing. Print orientation and support recommendations are
  noted in the CAD folder README.

### [ROCKET-8] Cardboard tube fuselage
Fuselage should be made out of cardboard tube: light, affordable, easily modified.

- **Rationale:** Estes/LOC-compatible body tubes are the standard in model
  rocketry. Cheap, predictable, easily cut and reinforced.
- **Verification:** Fuselage material is documented in the BOM. Tube diameter
  and wall thickness match a commercial product line (e.g. LOC Precision,
  Estes).

---

## Software subsystem

### [SW-1] On-board sensors
The system must contain sensors on-board.

- **Rationale:** Sensor data is the foundation of flight-state detection
  (apogee, deployment trigger), telemetry, and post-flight analysis.
- **Verification:** The `Sensor` trait and its implementations in
  `flight-computer/src/interfaces/sensor.rs` compile for the target MCU.
  Each sensor variant (altimeter, GPS, IMU) has a corresponding test.

#### [SW-1A] Sensor types
The sensors must include: IMU, altimeter/barometer, GPS.

- **Rationale:** IMU provides attitude and acceleration; altimeter provides
  altitude for apogee detection; GPS provides position for recovery. These
  three data sources cover the FSM's input requirements.
- **Verification:** `flight-computer` crate contains task definitions for
  `AltimeterSensor`, `GpsSensor`, and `ImuSensor`. `TOPICS_SIM_IN_LIST` in
  `proto/` includes matching `SimAltimeterTopic`, `SimGpsTopic`, `SimImuTopic`.

### [SW-2] Apogee recovery activation
The system must activate the recovery system at apogee.

- **Rationale:** Deploying at apogee minimises descent time and drift,
  maximising the chance of landing within the intended recovery zone.
- **Verification:** Flight test log shows deployment at or within 1 second of
  apogee (measured by altitude delta). SITL test confirms FSM reaches
  `Coast` → `DrogueDescent` transition at simulated apogee.

#### [SW-2A] Apogee detection
The system needs to detect apogee.

- **Rationale:** Without active apogee detection, deployment cannot be timed
  correctly. A timer-based fallback is less reliable.
- **Verification:** `flight-computer/src/core/state_machine/detectors/apogee_detector.rs`
  implements the configured apogee detection algorithm. Unit tests verify it
  triggers on a max-altitude data sequence and does not trigger on a
  non-apogee sequence.

### [SW-3] Human arming
The system must be armed by a human.

- **Rationale:** Prevents accidental activation during handling, transport,
  or pre-flight preparation. A positive arming action is a safety requirement.
- **Verification:** FSM initial state is `PreArmed`. Transition to `Armed`
  requires an external signal (`ArmingSystem::wait_arm`). SITL test confirms
  the FSM does not advance past `PreArmed` without the arming signal.

#### [SW-3A] Arming enables detectors
Arming the rocket enables the flight state detector, and recovery systems.

- **Rationale:** Detectors and recovery must be inactive before arming to
  prevent premature activation (e.g. from bench testing).
- **Verification:** FSM transitions `PreArmed` → `Armed` only after
  `ArmingSystem::wait_arm` resolves. The apogee and landing detectors are
  inactive before this transition.

#### [SW-3B] Arming status feedback
The system must provide feedback to the user about the arming status.

- **Rationale:** The operator must know whether the rocket is live. Ambiguous
  arming status has caused incidents.
- **Verification:** At least one of the following exists: LED pattern on the
  avionics board indicates armed vs pre-armed; GS telemetry displays the
  current `FlightState`; the stored flight log records the arm event.

### [SW-4] Persistent data storage and display
The system must display and store persistently all sensor data, events, and errors.

- **Rationale:** Post-flight analysis, failure diagnosis, and performance
  evaluation all depend on complete flight data. Real-time display enables
  ground-station monitoring.
- **Verification:** `storage_task` writes `Record`s to `FileSystem` during
  flight. `groundstation_task` publishes telemetry on `fc-gs.sock`. GS can
  display received records.

#### [SW-4A] Non-volatile storage
The storage must be non-volatile memory (e.g., Storage, EEPROM, Flash memory).

- **Rationale:** Flight data must survive power loss (battery disconnect,
  crash impact). Volatile memory loses the data on reset.
- **Verification:** `FileSystem` trait implementation in HW mode targets
  SD/flash. HOST mode targets host filesystem as an acceptable substitute
  for testing.

#### [SW-4B] Downloadable data
The system must provide a way to download the stored data after flight.

- **Rationale:** The operator needs to extract data from the recovered rocket
  for analysis. USB or radio download is the standard method.
- **Verification:** A post-flight download procedure exists (USB mass storage,
  CLI tool, or GS replay). Test flight data can be transferred to a host
  machine.

#### [SW-4C] Real-time telemetry display
The system must provide a real-time display of sensor data while in flight.

- **Rationale:** The operator needs live situational awareness — altitude,
  velocity, flight state — to monitor the flight and intervene if possible.
- **Verification:** GS (when connected) displays live telemetry from `Record`s
  received on `fc-gs.sock`. Update rate is sufficient for operator monitoring
  (≥ 1 Hz).

### [SW-5] Test suite
The system must provide a test suite: hardware tests, unit tests, flight
sequence tests, full system tests: testing successful and unsuccessful scenarios.

- **Rationale:** Systematic testing is the primary risk-reduction strategy
  for flight software. Without a test suite, regressions are discovered at
  the launch pad.
- **Verification:** `cargo nextest run` passes for all workspace crates.
  Test coverage includes unit tests, integration tests, and flight-scenario
  SITL tests.

#### [SW-5A] SITL simulations
The test suite must provide SITL (Software In The Loop) simulations, for
testing flight sequences and FC behavior without hardware.

- **Rationale:** SITL is the fastest feedback loop for FC development. Tests
  run in CI without special hardware, catching regressions before hardware
  testing.
- **Verification:** `simulator` crate publishes simulated sensor data over
  a transport. `flight-computer-host` crate runs the FC library against
  simulated peripherals. A scripted scenario can drive the full FSM cycle.

##### [SW-5A1] Simulated sensor data
The SITL simulations must provide a way to simulate sensor data inputs.

- **Rationale:** The FC consumes sensor data through the `Sensor` trait. SITL
  must inject realistic data to exercise the FSM.
- **Verification:** `SimAltimeter`, `SimGps`, `SimImu` implementations exist
  in `flight-computer/src/interfaces/impls/simulation/`. The simulator crate
  publishes matching Topics on `fc-sim.sock`.

##### [SW-5A2] Simulated recovery activation
The SITL simulations must provide a way to simulate recovery system activation.

- **Rationale:** The deployment system is a critical output path. SITL must
  confirm the FC issues the deploy command and the simulator observes it.
- **Verification:** `SimRecovery` implementation captures `deploy()` calls.
  The simulator subscribes to `SimDeploymentTopic` on `fc-sim.sock` and
  logs the event.

##### [SW-5A3] Simulated flight phases
The SITL simulations must provide a way to simulate different flight phases.

- **Rationale:** The FSM must be tested through every state (PreArmed, Armed,
  Boost, Coast, DrogueDescent, MainDescent, Touchdown). Each phase requires
  different sensor input patterns.
- **Verification:** Simulator scripted scenarios can sequence force events
  (thrust, drag, recovery) to drive the FC through the full FSM cycle.
  Scenario files define phase timing and triggers.

##### [SW-5A4] Simulated errors and failures
The SITL simulations must provide a way to simulate errors and failures.

- **Rationale:** The FC must handle sensor dropouts, deployment failures,
  and other off-nominal conditions without crashing. Error-path logic can
  only be tested by injecting failures.
- **Verification:** Simulator can inject sensor timeouts, invalid data
  values, or disconnect scenarios. FC error handling (timeouts, retries)
  is exercised in SITL tests.

##### [SW-5A5] Simulated flight data logging
The SITL simulations must provide a way to log simulated flight data.

- **Rationale:** SITL runs produce data for post-run analysis and regression
  comparison. Without logging, a failed run leaves no trace.
- **Verification:** Simulator writes structured logs (per-level JSON, internal
  tick log). FC storage task writes `Record`s in SITL mode. GS (when present)
  stores records per session.

##### [SW-5A6] Flight data visualization
The SITL simulations must provide a way to visualize flight data (e.g.,
graphs, charts).

- **Rationale:** Human review of raw telemetry is slow and error-prone.
  Visualization accelerates debugging and scenario validation.
- **Verification:** Simulator TUI displays live physics state, active forces,
  actuator status, and log tail. GS frontend (future) provides richer
  visualisation.

#### [SW-5B] Hardware mocking
The test suite must provide a way to mock hardware components.

- **Rationale:** Unit tests and SITL both require the FC library to run
  without real hardware. The `Sensor`, `ArmingSystem`, `DeploymentSystem`,
  `Led`, `FileSystem` traits enable this.
- **Verification:** Each peripheral trait has a simulation/ test
  implementation in `flight-computer/src/interfaces/impls/`. Trait consumer
  tests run against these implementations in `cargo nextest run`.

#### [SW-5C] Automated tests
The test suite must provide a way to run tests automatically.

- **Rationale:** CI and pre-commit hooks require a single command that runs
  the full suite. Manual test execution doesn't scale.
- **Verification:** `cargo nextest run --workspace` from the `code/` directory
  passes. CI configuration (GitHub Actions or equivalent) runs this on every
  push.

#### [SW-5D] Code coverage reports
The test suite must provide code coverage reports.

- **Rationale:** Coverage data identifies untested code paths and guides test
  investment. Without it, testing effort is blind.
- **Verification:** CI produces coverage output (e.g. `tarpaulin` or
  `grcov`). A coverage report is reviewable as a CI artifact.

#### [SW-5E] Performance benchmarks
The test suite must provide performance benchmarks.

- **Rationale:** Sensor rates, FSM transition latency, and storage throughput
  must stay within known bounds. Benchmarks catch regressions before launch.
- **Verification:** Benchmark harness exists (e.g. `criterion` or custom
  timing). Key metrics (sensor read time, FSM tick time, record write
  latency) are tracked and compared against a baseline.

### [SW-6] Modular design
The code should be modular and allow for future expansion, as new missions
and requirements are added to the project.

- **Rationale:** The project will evolve across multiple launches. A modular
  architecture (trait abstractions, crate boundaries, feature flags) makes
  change safe and local.
- **Verification:** FC library is `no_std` and uses traits for all peripheral
  interaction. Crates have single responsibilities. Feature flags gate
  target-specific code. New flight phases can be added as FSM states without
  rewriting existing states.

#### [SW-6A] Documented public API
The Public API for crates used should be documented.

- **Rationale:** Without documentation, consumers guess at interface contracts.
  Rustdoc is the standard mechanism.
- **Verification:** All public items in workspace crates have rustdoc comments.
  `cargo doc --workspace --no-deps` produces no文档 warnings.

#### [SW-6B] Hardware-agnostic flight computer
The flight computer code should be hardware-agnostic, allowing for easy
migration to different hardware platforms in the future.

- **Rationale:** The MCU target may change between revisions. Platform-specific
  code (drivers, HAL, linker script) must be replaceable without touching
  flight logic.
- **Verification:** `flight-computer` library crate has no `cfg(target_arch)`
  or `cfg(target_os)` in its core logic. All platform-specific code lives in
  `interfaces/impls/` and is selected by feature flag at link time.

### [SW-7] Rust programming language
The system should be developed using Rust programming language.

- **Rationale:** Rust provides memory safety, zero-cost abstractions, and
  `no_std` embedded support — a combination well-suited to flight software
  where reliability is critical.
- **Verification:** The workspace root `Cargo.toml` specifies Rust edition
  2024. All workspace crates compile with the stable toolchain. No C or C++
  source files exist in `code/`.

### [SW-8] LED status indicators
The system should display status information using LEDs.

- **Rationale:** LEDs provide immediate visual feedback without a display or
  GS connection. Essential for bench testing and pre-flight checks.
- **Verification:** `Led` trait in `flight-computer/src/interfaces/led.rs`
  defines the indicator API. LED implementations exist for the target board.

#### [SW-8A] Status encoding via LED patterns
The system should use different LED colors or patterns to indicate different
statuses (e.g., armed, flight phase, errors).

- **Rationale:** A single on/off LED carries one bit of information. Color
  and blinking patterns encode more states (armed, boost, coast, error) on
  limited hardware.
- **Verification:** LED pattern table is documented (in code comments or a
  design note). SITL display shows LED patterns alongside flight state for
  cross-reference.

#### [SW-8B] Per-component LED status
The system should use an LED per component to indicate its status and events
(e.g., sensor data valid, recovery system armed, flight state detected, data
logging active).

- **Rationale:** Per-component LEDs give immediate subsystem-level health
  (sensor OK, storage OK, armed) at a glance, without navigating menus or
  parsing logs.
- **Verification:** The `proto` crate defines distinct `LedStatus` Topics
  (`SimAltimeterLedTopic`, `SimGpsLedTopic`, etc.) for each component. The
  simulator TUI displays each LED independently.

### [SW-9] Ground-station communication protocol
The system must implement a ground-station communication protocol.

- **Rationale:** Two-way communication with the ground enables real-time
  telemetry, command-and-control, and post-flight data retrieval.
- **Verification:** FC and GS communicate over postcard-rpc Topics/Endpoints
  defined in `proto/`. `fc-gs.sock` carries telemetry (FC → GS) and commands
  (GS → FC). A GS connection is optional for FC operation.

#### [SW-9A] Real-time telemetry
The system must transmit real-time telemetry data to the ground station
during flight.

- **Rationale:** The operator needs live data for situational awareness and
  to detect anomalies as they happen.
- **Verification:** `groundstation_task` publishes `RecordTopic` messages
  on `fc-gs.sock` during flight. GS receives and stores them. SITL tests
  confirm telemetry flows end-to-end.

#### [SW-9B] Ground-station commands
The system must receive commands from the ground station (e.g., arm/disarm,
request data).

- **Rationale:** GS commands enable operator intervention during pre-flight
  and post-flight phases. In-flight commands are a future capability.
- **Verification:** `PingEndpoint` and `GlobalTickHzEndpoint` are defined in
  `proto/` and handled by the FC's postcard server. GS can issue ping
  requests and receive responses.

#### [SW-9C] Telemetry logging
The system must log all transmitted and received data for post-flight analysis.

- **Rationale:** A log of every message that crossed the GS ↔ FC boundary is
  essential for diagnosing communication issues and correlating with onboard
  records.
- **Verification:** GS stores received records to disk (one file per session).
  The format (newline-delimited JSON or equivalent) is documented and
  machine-readable.

#### [SW-9D] SITL and HITL usage
This system must also be used for SITL and HITL testing, for sending,
receiving, logging, and visualizing telemetry and simulated data.

- **Rationale:** Using the same GS implementation for flight, SITL, and HITL
  eliminates a class of integration bugs — the operator interface is identical
  regardless of test mode.
- **Verification:** Same `ground-station-backend` binary connects to FC
  in HOST mode (SITL), PIL mode (HITL), and HW mode (production flight).
  The GS REST API and storage path are mode-agnostic.
