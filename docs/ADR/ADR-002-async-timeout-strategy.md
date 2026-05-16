# ADR-002: Async timeout strategy for infinite loop error paths

- **Status:** Draft
- **Date:** 2026-05-09

> The timeout contracts this decision produced — per-domain timeouts, mapping table, and code changes — are tracked in [`../ROADMAP.md`](../ROADMAP.md). This ADR captures only *why* this approach was chosen over the alternatives, and *why each candidate was handled as it was*.

---

## Context

The FC library has several infinite `loop {}` bodies in both tasks (`sensor_task`, `storage_task`, `groundstation_task`, `postcard_server_task`) and state machine states (`wait_arm`, `await_deployment_system`, `await_apogee`, `await_touchdown`). Each loop contains `.await` calls that could theoretically hang indefinitely:

- A sensor's `parse_new_data()` may never return if the hardware or transport stalls.
- A filesystem `append_record()` or `flush()` may never complete if the storage medium blocks.
- A postcard `publish()` may never complete if the transport is wedged.
- A `deploy()` call may never return if the deployment hardware (or its sim proxy) does not respond.

The architecture imposes a hard constraint (from [`spec.md §6.5`](../software/spec.md#65-task-lifecycle-setup-vs-loop)):

> **Loop bodies must never panic.** Every error is logged and execution continues.

A hanging `.await` violates this constraint as surely as a panic does — the task stops making progress, and any subsystem that depends on it (FSM waiting on altitude data, storage backlog growing, ground station silent) degrades or stalls.

We need a mechanism to convert "this future might never complete" into a bounded, observable error that the loop can handle and continue past. `embassy_time::with_timeout` provides exactly this: it wraps any future and returns `Result<T, TimeoutError>`, cancelling the inner future on timeout.

However, `with_timeout` is not a hammer for every nail. We must distinguish between:

- **Error-path protection** — the future should normally complete quickly; if it doesn't, something is wrong and we should log, skip, and continue. → `with_timeout`.
- **Periodic polling** — the loop is designed to periodically check a condition regardless of whether the awaited event has happened. → `select` + `Ticker`, not `with_timeout`.

Mixing these up produces wrong behaviour: wrapping a polling future (like "wait for arm button, but keep blinking the LED") in a 2-second `with_timeout` changes the LED blink rate from 10 Hz to 0.5 Hz.

---

## Options considered

### Option 1 — Global `OPERATION_TIMEOUT` constant

A single `Duration::from_secs(2)` constant used everywhere.

**Rejected.** Different operations have fundamentally different timing requirements:
- Sensor reads at 50 Hz time out in ~30 ms (1.5 ticks), not 2 s.
- File writes at 576-byte granularity time out in 2 s, not 30 ms.
- Deployment retries time out per-attempt at 1 s, not 2 s.
- Storage flush timeout matches the write timeout at 2 s, not something else.

A single value over-constrains fast operations and under-constrains slow ones. **Adopted: per-domain timeout constants in each config struct**, following the existing `WAITING_ARM_INTERVAL` pattern.

### Option 2 — `with_timeout` for pre_armed (replace `select` + `Ticker`)

Replacing `select(self.arm_button.wait_arm(), ticker.next())` with `with_timeout(2s, self.arm_button.wait_arm())` changes the LED blink from 10 Hz to 0.5 Hz — barely visible and defeats the purpose of a status indicator.

**Rejected.** The `select` + `Ticker` pattern in `pre_armed` is periodic polling (blink LED while waiting for arm), not error-path protection. These are different patterns that should use different mechanisms.

### Option 3 — `with_retries` utility

A generic retry helper: `with_retries(fn, count, timeout) -> Result<T, RetryError>`.

**Deferred.** Only one use case (deployment retry loop) currently exists. A general utility adds API surface with no second consumer. The deployment retry logic is simple enough to write inline; extraction can happen when a second pattern emerges.

---

## Decision

Adopt `embassy_time::with_timeout` for error-path protection in all infinite loop `.await` calls where a hang would stall the loop. Timeout values are per-domain constants in their respective config structs.

The following table is the complete mapping:

| # | File | Operation | Loop type | Timeout | On timeout | Reasoning |
|---|---|---|---|---|---|---|
| 1 | `tasks/sensor.rs:18` | `sensor.parse_new_data()` | `loop`/`join` | `TICK_INTERVAL + (TICK_INTERVAL / 2)` | Log error, skip iteration | Sensor data read; if sensor stalls, the task stops producing records. 1.5× tick interval gives the sensor some slack while bounding the wait. Wrapped as `join(ticker.next(), with_timeout(timeout, parse_new_data()))` to retain concurrent ticker + read. |
| 2 | `tasks/storage.rs:65` | `storage.append_record()` | `loop` | `StorageConfig::WRITE_TIMEOUT = 2s` | Log error, continue | File I/O. Record already consumed from channel — cannot retry, best to log and move on. Partial write bounded by `WRITE_BUFFER_SIZE = 576`. |
| 3 | `tasks/storage.rs:69` | `storage.flush()` | `loop` | `StorageConfig::FLUSH_TIMEOUT = 2s` | Log warning, continue | File sync. Bounded data loss if cancelled (unflushed bytes). |
| 4 | `tasks/groundstation.rs:60` | `send_to_ground_station(state)` | `loop` | `GroundStationConfig::PUBLISH_TIMEOUT = 2s` | Log error, continue | Postcard publish; best-effort semantics. Transport corruption causes reconnect which is handled transparently by postcard server. |
| 5 | `tasks/groundstation.rs:69` | `send_to_ground_station(record)` | `loop` | Same as #4 | Same | Same analysis as #4. |
| 6 | `core/state_machine/states/armed.rs:16` | `deployment_system.deploy()` | `loop` (retry) | `Duration::from_secs(1)` | Log error, retry (timeout provides the 1s pacing, replaces `Timer::after_secs(1)`) | Mission-critical. Per-attempt timeout protects against hung deploy while maintaining ~1 attempt/s rate. |
| 7 | `core/state_machine/detectors/apogee_detector.rs:72` | `wait_new_data_and_update_buffers()` | `loop`/ticker | `DETECTOR_TICK_INTERVAL / 2` | Log error, skip iteration | Data wait. Inner `LATEST_ALTITUDE_SIGNAL.wait()` is unbounded if altimeter stops publishing. Half-tick timeout bounds it. |
| 8 | `core/state_machine/detectors/touchdown_detector.rs:62` | `wait_new_data_and_update_buffers()` | `loop`/ticker | `DETECTOR_TICK_INTERVAL / 2` | Log error, skip iteration | Same as #7. |

### Not applied

| File | Operation | Reason |
|---|---|---|
| `tasks/postcard.rs:78` | `server.run()` | Only returns on error. A healthy connection never times out. No benefit. |
| `core/state_machine/states/pre_armed.rs:44` | `select(wait_arm(), ticker.next())` | Periodic polling pattern (LED blink), not error-path protection. Keep `select` + `Ticker`. |

---

## Cancellation safety analysis

`embassy_time::with_timeout` works by racing the inner future against a timer. If the timer wins, the inner future is dropped (cancelled). The safety of cancellation depends on what that inner future does.

### `embassy_sync::signal::Signal::wait()` — used by sensor reads, altitude signal, detector data waits

**Cancellation-safe.** When a `wait()` future is dropped, the signal retains the state it was in (`Waiting(waker)` or `Signaled(val)`). A stale `Waker` is left behind but executors handle stale wakers gracefully (no-op). The signalled value, if any, survives in `State::Signaled(T)` and is delivered to the next `wait()` call.

**Verdict:** Safe for all signal-based operations.

### `postcard_rpc::Sender::publish()` — used by groundstation sends

**Partially safe.** Cancelling mid-publish may leave partial serialised data in the transport buffer. The receiver detects a framing error on the next read (postcard-rpc uses length-prefixed framing), disconnects, and triggers a reconnect cycle via `server.run()`'s error return.

**Verdict:** Practically safe — the transport recovers via reconnect. The cancelled send is lost, but groundstation sends are best-effort (function already swallows errors).

### Host `tokio::fs` file operations — used by storage

**Partially safe.** If `write_all()` is cancelled mid-write, the OS file position advances by whatever bytes were written before cancellation. The next write appends at the wrong offset, producing a garbled record. The damage is bounded by `WRITE_BUFFER_SIZE = 576` bytes.

**Verdict:** Acceptable for flight data logging — a garbled record is far better than a stalled storage task that blocks all subsequent records. The storage session uses unique filenames per run, so corruption does not cascade.

### `switch_hal::OutputSwitch::on()` (embedded deployment)

**Cancellation-safe.** A GPIO register write is effectively instant; the future resolves on the first poll. Cancellation would not occur in practice.

**Verdict:** Safe.

### `postcard_rpc::Server::run()` — NOT applied per decision above

No analysis needed.

---

## DeploymentSystem trait changes

The `DeploymentSystem` trait gains a new required method:

```rust
pub trait DeploymentSystem {
    type Error: core::fmt::Debug;

    /// Trigger deployment. Returns Ok(()) on successful trigger.
    async fn deploy(&mut self) -> Result<(), Self::Error>;

    /// Verify deployment was successful. Returns Ok(true) if deployed.
    /// Must be implemented by all impls (no default).
    async fn verify_deployment(&mut self) -> Result<bool, Self::Error>;
}
```

**No default implementation.** Every impl must explicitly handle verification:

- **Simulation** (`SimRecovery`): Sends deploy signal, then awaits acknowledgment from the simulator via the postcard response on the deployment topic. Returns `Ok(true)` if acknowledged, `Ok(false)` if not.
- **Embedded** (`DeploymentSwitch`): `unimplemented!()` for now. Hardware-level verification (continuity check, current draw sensing) will be added when the HW deployment system is designed.

The `await_deployment_system` loop in `armed.rs` changes to:

```rust
loop {
    match with_timeout(1s, deployment_system.deploy()).await {
        Ok(Ok(())) → verify → break on verification, retry on failure
        Ok(Err(e)) → log, retry
        Err(_)     → log timeout, retry
    }
}
```

The `Timer::after_secs(1)` retry delay is removed — the `with_timeout(1s, ...)` provides the same pacing (one attempt per second), while also protecting against a hung `deploy()`.

The `verify_deployment()` call itself should also be timeout-protected, with a shorter timeout (e.g. 500 ms).

### Simulation error type change

`SimRecovery`'s error type changes from `Infallible` to a type that can represent publish failures:

```rust
type Error = DeploymentError<postcard_rpc::wire::WireError>;
```

(Exact error type depends on postcard-rpc's wire error type.) When `publish()` fails, the sim implementation returns `Err` instead of silently logging and returning `Ok`.

---

## Consequences

### Config additions (`config.rs`)

| Config struct | New constant | Value |
|---|---|---|
| `StorageConfig` | `WRITE_TIMEOUT` | `Duration::from_secs(2)` |
| `StorageConfig` | `FLUSH_TIMEOUT` | `Duration::from_secs(2)` |
| `GroundStationConfig` | `PUBLISH_TIMEOUT` | `Duration::from_secs(2)` |

No global `OPERATION_TIMEOUT` is added. Each config struct owns its own timeout.

Sensor read and detector timeouts are computed at the call site from `TICK_INTERVAL / 2` or `TICK_INTERVAL + (TICK_INTERVAL / 2)` — no new config constants needed.

### Code changes

| File | Change |
|---|---|
| `config.rs` | Add `WRITE_TIMEOUT`, `FLUSH_TIMEOUT` to `StorageConfig`; add `PUBLISH_TIMEOUT` to `GroundStationConfig`. |
| `tasks/sensor.rs` | Wrap `parse_new_data()` in `with_timeout(Self::TICK_INTERVAL + (Self::TICK_INTERVAL / 2), ...)` inside the existing `join()`. |
| `tasks/storage.rs` | Wrap `append_record()` and `flush()` in `with_timeout(...)`. On timeout: log and continue. |
| `tasks/groundstation.rs` | Wrap both `send_to_ground_station()` calls in `with_timeout(...)`. On timeout: log and continue. |
| `core/state_machine/detectors/apogee_detector.rs` | Wrap `wait_new_data_and_update_buffers()` in `with_timeout(DETECTOR_TICK_INTERVAL / 2, ...)`. On timeout: `continue` the ticker loop. |
| `core/state_machine/detectors/touchdown_detector.rs` | Same as apogee. |
| `core/state_machine/states/armed.rs` | Replace `deploy().await` + `Timer::after_secs(1)` with `with_timeout(1s, deploy()).await`. Add `verify_deployment()` step. |
| `interfaces/deployment_system.rs` | Add `verify_deployment()` required method. |
| `interfaces/impls/simulation/deployment_system.rs` | Change error type from `Infallible`; return `Err` on publish failure; implement `verify_deployment()`. |
| `interfaces/impls/embedded/deployment_switch.rs` | Implement `verify_deployment()` as `unimplemented!()` with a doc comment explaining why. |

### Excluded

- `tasks/postcard.rs` — no change (`server.run()` only returns on error).
- `core/state_machine/states/pre_armed.rs` — no change (`select` + `Ticker` is the correct pattern for polling).

---

## See also

- [`../software/spec.md` §6.5](../software/spec.md#65-task-lifecycle-setup-vs-loop) — "Loop bodies must never panic" invariant.
- [`../software/spec.md` §6.6](../software/spec.md#66-task-orchestration) — Cancellation safety and orchestration shape.
- [`../ROADMAP.md`](../ROADMAP.md) — Implementation tasks for this ADR.
- [`../../code/flight-computer/src/config.rs`](../../code/flight-computer/src/config.rs) — Config structs where timeout constants will be added.
