# Testing Strategy

- **Status:** draft
- **Date:** 2026-05-27

## Purpose

Define a three-tier testing architecture for the entire project — unit, integration,
and cross-crate — with clear ownership, location conventions, and quality gates.
Every crate and every type of test (functional, regression, HW-in-the-loop, tracing,
benchmark, static analysis, coverage) has a home in this model.

## Motivation

The codebase has grown organically from a single prototype into a multi-crate Rust
workspace with embedded, host, and simulation targets. Testing has not kept pace:

- Existing `#[cfg(test)]` blocks are ad-hoc and concentrated in `flight-computer`
- No integration test directories (`tests/`) exist anywhere
- No cross-crate tests exist
- The simulator crate has zero tests
- HW binary crates have no structured test pattern
- No CI enforces test quality

A systematic testing strategy is the primary risk-reduction lever for flight software
(per [SW-5]) and a prerequisite for CI, coverage [SW-5D], and performance
benchmarking [SW-5E].

## Scope

**In scope:** All software in `code/`, including workspace crates and embedded binary
crates (`cross-esp32-s3`, `cross-nucleo-f413zh`). CI configuration. Code coverage
tooling. Performance benchmarking.

**Out of scope:** Hardware-in-the-loop that requires test equipment beyond the MCU
itself (e.g. vacuum chambers for altimeter testing). Mechanical testing. Recovery
system functional testing. Those are governed by [DEV-3] and [DEV-4].

---

## 1. The three-tier model

| Tier | Location | Test runner | Scope | Typical test |
|---|---|---|---|---|
| **Unit** | `#[cfg(test)]` in `src/` | `cargo nextest` | Single module or struct — no external deps beyond mocks | "apogee detector triggers on descending-altitude sequence" |
| **Integration** | `code/<crate>/tests/` | `cargo nextest` | Crate's public API — multiple modules, task lifecycles, in-process transports | "full flight scenario with mock peripherals transitions PreArmed → Touchdown" |
| **Cross-crate** | `code/tests/` (workspace member) | `xtask test-cross-crate` | Multiple crates — process boundaries, IPC, protocol conformance | "simulator publish → FC-host process → GS receives telemetry" |

Every tier runs in CI. A failure in any tier is a build failure.

### 1.1 Unit tests (`#[cfg(test)]`)

**Focus:** Implementation correctness, data invariants, regression tests.

Rules:
- No I/O, no filesystem, no network, no process spawning.
- External dependencies (sensors, filesystem, timers) are mocked via the existing
  trait system (`Sensor`, `FileSystem`, `Led`, `ArmingSystem`, `DeploymentSystem`).
- `embassy-time` uses the mock-driver (already configured in dev-dependencies).
- Property-based tests (`proptest`, `quickcheck`) belong here for state-machine
  invariants and serialization round-trips.

### 1.2 Integration tests (`code/<crate>/tests/`)

**Focus:** Behavior — "under X situation, if Y happens, Z should occur."

Each file in `tests/` is a separate binary that links the crate as an external
dependency. Rules:
- Use the crate's public API only (same as any consumer).
- In-process transport (`transport-thread` from `proto`) preferred — no process
  boundaries, faster execution, deterministic.
- Flight scenarios use the same postcard-rpc endpoints/topics as production but
  over thread channels instead of IPC sockets.
- Each test file exercises one behavioral area (e.g. `fsm_scenarios.rs`,
  `sensor_timeouts.rs`, `storage_lifecycle.rs`).
- Logging via `test-log` + `tracing-subscriber` for debugging test failures.

### 1.3 Cross-crate tests (`code/tests/`)

**Focus:** Glue between components. Process-level behavior. Protocol conformance.

A dedicated workspace member at `code/tests/` containing integration test binaries
that exercise real process boundaries:

- `fc_sim_ipc.rs` — FC-host ↔ Simulator over `fc-sim.sock`
- `fc_gs_ipc.rs` — FC-host ↔ GS backend over `fc-gs.sock`
- `full_stack_sitl.rs` — All three processes, full flight scenario
- `disconnect_handling.rs` — Peer drops, no panics
- `protocol_compat.rs` — Message format consistency across crate versions

These tests spawn child processes, connect sockets, and assert behavior with
timeouts. The test harness is a reusable library within the `tests` crate (see
implementation details deferred — §6).

---

## 2. Per-crate test inventory

### 2.1 `proto` (no_std, wire vocabulary)

| Tier | What | Priority |
|---|---|---|
| **Unit** | Newtype construction/conversion/arithmetic | High |
| **Unit** | serde round-trips (postcard encode → decode → assert equal) | High |
| **Unit** | Topic/Endpoint path constants are correct | Medium |
| **Unit** | Enum variant construction helpers | Low |
| **Integration** | `transport-thread` client/server handshake | High |
| **Integration** | Multiple endpoint registration and dispatch | Medium |
| **Integration** | Feature flag compilation matrix (`default`, `simulator-endpoints`, `host`, `pil`, `hw`) | Medium |
| **Cross-crate** | Proto types produced by FC are consumed by GS (wire compatibility) | High |

### 2.2 `flight-computer` (no_std lib, core of everything)

| Tier | What | Priority |
|---|---|---|
| **Unit** | FSM transitions: every valid state pair, every invalid transition rejected | **Critical** |
| **Unit** | Apogee detector: triggers on descending-altitude data, no-ops on ascent | **Critical** |
| **Unit** | Landing detector: triggers on sustained zero-velocity | **Critical** |
| **Unit** | Config struct construction and validation | Medium |
| **Unit** | Event/error type conversions | Low |
| **Unit** | Storage record serialization round-trips | Medium |
| **Unit** | LED pattern encoding | Low |
| **Unit** | `Ticker` behavior with mock embassy-time driver | High |
| **Unit** | `FileSystem` trait operation modeling | Medium |
| **Unit** | Regression tests for fixed bugs (added alongside fix) | **Critical** |
| **Integration** | Full task lifecycle: spawn sensors + FSM + storage + GS → graceful shutdown | **Critical** |
| **Integration** | Flight scenarios: PreArmed → Armed → Boost → Coast → Drogue → Main → Touchdown | **Critical** |
| **Integration** | Error injection: sensor timeout → FSM logs and continues | High |
| **Integration** | Error injection: storage write failure → FSM logs and continues | High |
| **Integration** | Error injection: GS send failure → FSM logs and continues | Medium |
| **Integration** | Panic-graceful: sensor task panic → other tasks continue | Medium |
| **Integration** | Feature flag combinations compile (`impl_sim`, `impl_host`, `log`, `defmt`) | High |
| **Cross-crate** | N/A — tested via binaries that consume this library | N/A |

#### 2.2.1 Embedded impl test helpers

The `interfaces/impls/embedded/` module provides real hardware driver implementations.
These cannot contain `#[test]` functions (they would link into no_std firmware).
Instead, each implementation file exposes `pub fn test_*()` functions that:

1. Take an initialized embedded-hal peripheral as input
2. Initialize the device
3. Perform a read cycle
4. Assert basic sanity (e.g. pressure > 0, temperature within expected range)
5. Return `Result<(), Error>`

The HW binary crates (`cross-esp32-s3`, `cross-nucleo-f413zh`) then call these
helpers from their own `#[test]` functions with real hardware. This keeps the
"test with real hardware" pattern out of the no_std library while making it
trivially reusable across MCU targets.

| File | Helper | What it validates |
|---|---|---|
| `embedded/sensor/bmp280.rs` | `pub fn test_bmp280<I, E>(i2c: I)` | Pressure read, temperature read, altitude calculation |
| `embedded/sensor/bno055.rs` | `pub fn test_bno055<I, E>(i2c: I)` | IMU orientation read |
| `embedded/sensor/gps.rs` | `pub fn test_gps(...)` | Serial read, NMEA parse |
| `embedded/sd_card.rs` | `pub fn test_sd_card(...)` | Block read/write |
| `embedded/deployment_switch.rs` | `pub fn test_switch(...)` | GPIO read |
| `embedded/arming_button.rs` | `pub fn test_button(...)` | GPIO read |
| `embedded/led_device.rs` | `pub fn test_led(...)` | GPIO set |

These are **not** run in CI (no HW attached). They are manual or pre-flight checks
only. Marking convention: `#[cfg(feature = "hw_test")]` or a similar opt-in flag
to prevent accidental inclusion in release builds.

### 2.3 `simulator` (std, physics engine + IPC client)

| Tier | What | Priority |
|---|---|---|
| **Unit** | 1D parabolic physics: burn → coast → apogee → descent | **Critical** |
| **Unit** | Force composition: thrust + drag + gravity = net acceleration | High |
| **Unit** | Scripted event parsing and timeline computation | High |
| **Unit** | Config struct defaults and validation | Medium |
| **Integration** | Full scenario playback → FC state machine transitions correctly | **Critical** |
| **Integration** | Postcard-rpc client connect/publish/subscribe over thread transport | High |
| **Integration** | Log output contains expected events from scenario | Medium |
| **Integration** | TUI state model (if separable from rendering) | Low |
| **Cross-crate** | Simulator + FC-host process: sensor publish → FC responds | **Critical** |
| **Cross-crate** | Simulator disconnects → FC-host does not panic | High |
| **Cross-crate** | FC-host disconnects → simulator does not panic | High |

### 2.4 `flight-computer-host` (std, thin binary)

| Tier | What | Priority |
|---|---|---|
| **Unit** | Socket path construction (OS-dependent) | Low |
| **Integration** | Socket bind, accept sim connection, accept GS connection | High |
| **Integration** | Sim connect timeout → graceful retry or exit | Medium |
| **Integration** | FC startup sequence → all tasks running | Medium |
| **Cross-crate** | Full SITL run: FC-host + simulator + GS as co-processes | **Critical** |

### 2.5 `ground-station-backend` (std, Rocket server)

| Tier | What | Priority |
|---|---|---|
| **Unit** | Record filtering logic | Medium |
| **Unit** | API response model construction | Low |
| **Integration** | REST endpoint behavior (`GET /api/status`, `GET /api/records`, etc.) | High |
| **Integration** | NDJSON session storage write + read-back | High |
| **Integration** | Postcard client subscription to `RecordTopic` | High |
| **Integration** | Error responses for invalid requests | Medium |
| **Cross-crate** | FC publishes telemetry → GS receives and stores | **Critical** |
| **Cross-crate** | GS sends ping → FC responds | High |
| **Cross-crate** | FC disconnects → GS degrades gracefully, no panic | High |

### 2.6 `xtask` (project task runner)

Minimal testing — single integration check that `cargo xtask <known_task>` does
not panic. Error paths are exercised manually during development.

---

## 3. Other test types

### 3.1 Tracing-based tests

**Integration tier.** Use `tracing-subscriber` with a test sink (`tracing-test` or
custom `TestWriter`) to assert that specific events fire in expected order during
scenario runs. Example: "FSM enters Coast state → `info!` log contains
'Transition: Boost → Coast'."

Each integration test can optionally enable a `TracingTestCase` helper that
captures events and allows assertions like:

```rust
assert_event!(logs, contains("Transition: Boost → Coast"));
assert_no_event!(logs, contains("panic"));
```

### 3.2 Flamegraph / profiling

**Integration tier** (crate-level benchmarks) + **Cross-crate tier** (system-level).

- **Crate level:** `criterion` benchmarks in `code/<crate>/benches/` for hot paths:
  sensor data parsing, FSM transition, postcard encode/decode, storage record
  serialization.
- **System level:** `pprof` or `flamegraph` on cross-crate scenario runs to identify
  bottlenecks in the end-to-end pipeline.

### 3.3 Performance benchmarks

**Integration tier.** Governed by [SW-5E]:

| Benchmark | Location | What it measures |
|---|---|---|
| FSM transition latency | `flight-computer/benches/` | Time to process one sensor tick |
| Sensor data parse | `flight-computer/benches/` | `parse_new_data()` throughput |
| Postcard encode/decode | `proto/benches/` | Wire format throughput |
| Storage write | `flight-computer/benches/` | `append_record()` latency |
| Physics step | `simulator/benches/` | One physics tick computation |

Baselines are tracked in a `benches/BASELINES.md` or similar. CI warns on
regression >10%.

### 3.4 Static analysis

**Cross-cutting gate, not a test tier.** Already partially configured:

| Crate | Lints |
|---|---|
| `flight-computer` | `unsafe_code = forbid`, clippy `pedantic + nursery + unwrap_used` |
| `proto` | `unsafe_code = forbid`, clippy `pedantic + nursery + unwrap_used` |
| Others | None (gap) |

Goal: All workspace crates have consistent clippy lints. CI runs `cargo clippy
--workspace --all-features --deny warnings`.

### 3.5 Code coverage

**Metric collected across all three tiers.** Governed by [SW-5D].

Tool: `grcov` (preferred for Rust, works with nextest). CI job runs unit +
integration + cross-crate tests under coverage instrumentation and produces an
HTML report + summary badge. Coverage targets (to be refined after baseline):
- Line coverage: ≥ 70% for `flight-computer`, ≥ 50% for other crates.
- No untested `unsafe` blocks.

### 3.6 Property-based tests

**Unit + Integration tiers.**

- **Unit:** `proptest` for FSM invariants — "for any valid sequence of sensor
  readings, the FSM remains in a valid state." Round-trip serialization properties.
- **Integration:** Behavior properties — "for any valid scenario script, the
  simulator produces monotonic timestamps." (Deferred until proptest infrastructure
  is proven at unit level.)

---

## 4. Embedded HW test pattern

```
┌─────────────────────────────────────────────────────────────────┐
│ flight-computer (no_std lib)                                     │
│                                                                  │
│  interfaces/impls/embedded/sensor/bmp280.rs                      │
│    pub fn test_bmp280<I, E>(i2c: I) -> Result<(), E>            │
│      where I: I2c<SevenBitAddress, Error = E>, E: Debug          │
│    ─────────────────────────────────────────────────────────     │
│    Initializes BMP280 from I2C, reads pressure + temperature,    │
│    asserts values in sane range, returns Ok/Err.                 │
│    NOT #[test] — just a public helper.                           │
│                                                                  │
│  (same pattern for bno055, gps, sd_card, switch, button, led)   │
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │ called by
┌─────────────────────────────────────────────────────────────────┐
│ cross-esp32-s3 (binary crate, HW target)                         │
│                                                                  │
│  tests/bmp280_hw_test.rs                                         │
│    #[test]                                                        │
│    fn test_bmp280_on_esp32() {                                    │
│        let i2c = esp_hal::i2c::I2C::new(...);                    │
│        flight_computer::interfaces::impls::embedded::             │
│            sensor::bmp280::test_bmp280(i2c).unwrap();            │
│    }                                                             │
│                                                                  │
│  Same test file can test the *same flight-computer helper*       │
│  but on different HW — ESP32 via SPI/I2C, STM32 via UART/...    │
└─────────────────────────────────────────────────────────────────┘
```

Key rules:
- Test helpers are `pub fn`, **not** `#[test]`, so they don't require std or
  a test runner in the no_std lib.
- HW binary crates enable `std` in their own `#[cfg(test)]` and call the helpers.
- Helpers are gated behind `#[cfg(any(test, feature = "hw_test"))]` in the FC lib
  to prevent bloat in release firmware.
- "Sanity range" constants (e.g. `EXPECTED_PRESSURE_MIN_HPA`, `EXPECTED_TEMP_MIN_C`)
  live in the test helper or a companion config module — not in production code
  paths.
- These tests are **not** CI-run (no HW in CI). They are manual launch-prep or
  bench tests. An xtask command (`cargo xtask test-hw --target esp32`) can provide
  a unified interface.

---

## 5. Gap analysis

### 5.1 Existing tests

| File | Crate | Tier | Notes |
|---|---|---|---|
| `src/lib.rs` | flight-computer | Unit | Module-level doc tests |
| `src/core/trace.rs` | flight-computer | Unit | Trace struct assertions |
| `src/sync.rs` | flight-computer | Unit | Synchronization primitive tests |
| `src/core/sensors/altimeter.rs` | flight-computer | Unit | Altitude-from-pressure logic |
| `src/interfaces/impls/simulation/sensor.rs` | flight-computer | Unit | Sim sensor impl tests |
| `src/interfaces/impls/simulation/arming_system.rs` | flight-computer | Unit | Sim arming impl tests |
| `proto/src/newtypes/ping.rs` | proto | Unit | Ping message newtype |
| `proto/src/newtypes/naive_time.rs` | proto | Unit | NaiveTime newtype |
| `proto/src/newtypes/fix_type.rs` | proto | Unit | FixType newtype |

**Total: 6 unit test modules in flight-computer, 3 in proto. Zero integration
tests. Zero cross-crate tests.**

### 5.2 Missing by crate

| Crate | Unit gap | Integration gap | Cross-crate gap |
|---|---|---|---|
| `proto` | Many newtypes, all topics/endpoints | No `tests/` dir | Wire compat not tested |
| `flight-computer` | FSM, detectors, config, storage, LED — all untested | No `tests/` dir | N/A |
| `simulator` | **Everything** — zero tests | No `tests/` dir | Not tested with FC |
| `flight-computer-host` | Minimal, but none exist | Socket lifecycle untested | Not tested with sim/GS |
| `ground-station-backend` | Minimal, but none exist | API, storage, client all untested | Not tested with FC |
| `xtask` | Acceptable gap | Acceptable gap | N/A |

### 5.3 Infrastructure gaps

| Capability | Status | Required by |
|---|---|---|
| CI (GitHub Actions) | Not configured | [SW-5C] |
| Coverage tooling | Not configured | [SW-5D] |
| Benchmark harness | Not configured | [SW-5E] |
| Consistent clippy across all crates | Only flight-computer + proto | Static analysis gate |
| `code/tests/` workspace member | Does not exist | Cross-crate tier |
| `hw_test` feature flag in flight-computer | Does not exist | HW test helpers |
| xtask `test-all` command | Does not exist | Unified test UX |
| xtask `test-hw` command | Does not exist | HW test UX |

---

## 6. Implementation roadmap

Testing work spans the whole codebase and will uncover improvements needed
in the code itself. The implementation order minimizes risk by building from
the foundation up.

### Phase 1: Foundation (unit tests)

Add the unit test scaffolding that the rest depends on. No new infrastructure,
no behavioral changes, no transport dependencies.

| # | Task | Deliverable |
|---|---|---|
| 1.1 | `hw_test` feature flag in `flight-computer/Cargo.toml` | Feature gate for HW test helpers |
| 1.2 | Unit tests for `flight-computer` FSM | Every state transition tested |
| 1.3 | Unit tests for apogee detector | Trigger + no-trigger cases |
| 1.4 | Unit tests for landing detector | Trigger + no-trigger cases |
| 1.5 | Unit tests for config structs | Validation, defaults, const correctness |
| 1.6 | Unit tests for storage records | Serialization round-trips |
| 1.7 | Unit tests for proto newtypes | Construction, conversion, serde |
| 1.8 | Unit tests for proto topics/endpoints | Path constants correctness |
| 1.9 | Unit tests for simulator physics | 1D trajectory math |
| 1.10 | Unit tests for simulator script engine | Event parsing, timeline |
| 1.11 | Extend clippy lints to all workspace crates | `Cargo.toml` lint tables |
| 1.12 | Add property-based tests for FSM invariants | `proptest` integration |

**Exit criteria:** `cargo nextest run --workspace` passes. `cargo clippy --workspace --all-features --deny warnings` passes.

### Phase 2: Behavior (integration tests)

Add `tests/` directories. Use in-process transports. Exercise real crate
behavior.

| # | Task | Deliverable |
|---|---|---|
| 2.1 | Integration tests for proto transport handlers | `transport-thread` handshake, dispatch |
| 2.2 | Integration tests for flight-computer task lifecycle | Spawn all tasks, run, shut down |
| 2.3 | Integration tests for full flight scenarios | PreArmed → Touchdown with mock peripherals |
| 2.4 | Integration tests for error injection | Sensor timeout, storage failure, GS failure |
| 2.5 | Integration tests for panic isolation | One task panics, others continue |
| 2.6 | Integration tests for simulator scenarios | Full scenario → FC transitions verified |
| 2.7 | Integration tests for GS backend REST API | Rocket `local::Client`, endpoint behavior |
| 2.8 | Integration tests for GS backend storage | NDJSON read/write |
| 2.9 | Integration tests for FC-host socket lifecycle | Bind, accept, timeout |
| 2.10 | Add `criterion` benchmarks for hot paths | `benches/` in each major crate |

**Exit criteria:** `cargo nextest run --workspace` includes integration tests.
Benchmarks compile and produce baseline numbers.

### Phase 3: System (cross-crate tests)

Create `code/tests/` workspace member. Spawn real processes. Test the glue.

| # | Task | Deliverable |
|---|---|---|
| 3.1 | Create `code/tests/` crate with harness library | Test helper: spawn process, connect socket, wait-for-condition, assert |
| 3.2 | Cross-crate test: FC-host ↔ simulator IPC | Sensor data flow, FSM transitions |
| 3.3 | Cross-crate test: FC-host ↔ GS IPC | Telemetry flow, ping command |
| 3.4 | Cross-crate test: full-stack SITL scenario | Three processes, full flight |
| 3.5 | Cross-crate test: disconnect handling | Peer drops → no panic on either side |
| 3.6 | Cross-crate test: protocol compatibility | All endpoints/topics compile and match |

**Exit criteria:** `cargo nextest run --workspace` + `xtask test-cross-crate` passes.

### Phase 4: Infrastructure & CI

| # | Task | Deliverable |
|---|---|---|
| 4.1 | GitHub Actions workflow: clippy + build + unit + integration | CI runs on every push/PR |
| 4.2 | GitHub Actions workflow: cross-crate tests | CI runs on every push/PR |
| 4.3 | Coverage instrumentation (`grcov`) | Coverage report per CI run |
| 4.4 | `xtask test-all` command | Runs all 3 tiers + clippy |
| 4.5 | `xtask test-hw` command | Runs HW test helpers (manual) |
| 4.6 | Benchmark regression tracking | CI warns on >10% regression |

**Exit criteria:** Green CI badge. Coverage reports published. `cargo xtask test-all` is the single command for full validation.

### Phase 5: Embedded HW tests (ongoing)

| # | Task | Deliverable |
|---|---|---|
| 5.1 | `test_bmp280()` helper in flight-computer | BMP280 I2C sanity test |
| 5.2 | `test_bno055()` helper | BNO055 I2C sanity test |
| 5.3 | `test_gps()` helper | GPS serial sanity test |
| 5.4 | `test_sd_card()` helper | SD block read/write sanity test |
| 5.5 | `test_switch()` / `test_button()` / `test_led()` helpers | GPIO sanity tests |
| 5.6 | HW binary crates call helpers from `#[test]` | `cross-esp32-s3/tests/*.rs` etc. |

**Exit criteria:** Each HW binary crate has `cargo test --features hw_test` that
exercises every onboard sensor when run on real hardware.

---

## 7. CI integration

### 7.1 Workflow: `ci.yml`

```yaml
name: CI

on: [push, pull_request]

jobs:
  # Fast fail — clippy first
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo clippy --workspace --all-features --deny warnings
        working-directory: code

  # Build all crates, all feature combinations
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --workspace --all-features
        working-directory: code

  # Unit + integration tests via nextest
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run --workspace
        working-directory: code

  # Cross-crate tests (separate job, longer timeout)
  test-cross-crate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@nextest
      - run: cargo nextest run -p cross-crate-tests
        working-directory: code

  # Coverage
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install grcov
      - run: |
          cargo clean
          CARGO_INCREMENTAL=0 RUSTFLAGS="-Cinstrument-coverage" \
            cargo nextest run --workspace
          grcov . --binary-path target/debug/ -s . -t html \
            --branch --ignore-not-existing -o coverage/
        working-directory: code
      - uses: actions/upload-artifact@v4
        with:
          name: coverage
          path: code/coverage/
```

---

## 8. Connection to existing requirements

| Requirement | How this strategy satisfies it |
|---|---|
| [SW-5] Test suite | Three tiers cover unit, integration, and system-level testing |
| [SW-5A] SITL simulations | Cross-crate tests exercise FC-host + simulator + GS in process topology |
| [SW-5A1–5A5] Simulated sensors, recovery, flight phases, errors, logging | Integration and cross-crate tests exercise each via scripted scenarios |
| [SW-5B] Hardware mocking | Unit and integration tests use trait-based mocks (already in place) |
| [SW-5C] Automated tests | CI runs all three tiers on every push |
| [SW-5D] Code coverage | `grcov` job in CI produces coverage reports |
| [SW-5E] Performance benchmarks | `criterion` benchmarks in each major crate |
| [SW-3] Human arming | Integration test asserts FSM does not leave PreArmed without arming signal |
| [SW-3A] Arming enables detectors | Integration test asserts detectors inactive before Armed |
| [SW-3B] Arming status feedback | Integration test asserts LED pattern or GS telemetry reflects armed state |

---

## 9. Open questions

- **Cross-crate test crate name:** `cross-crate-tests`? `system-tests`?
  `integration-tests`? (Avoiding confusion with Cargo's built-in `tests/` dir.)
- **Coverage target thresholds:** Need a baseline before setting hard numbers.
- **Benchmark CI integration:** `criterion` can output machine-readable data for
  trend tracking, but initial setup is manual comparison. What's the threshold for
  CI failure on regression?
- **Windows/macOS CI:** Currently targeting Linux only. Cross-platform gaps
  (IPC socket names, path separators) are known — add when needed.
- **`code/tests/` harness library:** spawn-process-with-timeout + socket-wait +
  log-collection utility. Implementation approach deferred per roadmap (see §6).
