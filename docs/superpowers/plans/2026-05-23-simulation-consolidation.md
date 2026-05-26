# Simulation Code Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Consolidate all `#[cfg(feature = "impl_sim")]` code into `simulation.rs` so postcard.rs and mod.rs no longer have scattered sim-related items.

**Architecture:** Move 4 sim dispatch handlers and `postcard_sim_server_task` from `postcard.rs::simulator`, and `flight_state_sim_publisher_task` from its own file, into `simulation.rs`. Update all import paths. `run_flight_computer` is already correct.

**Tech Stack:** Rust, postcard-rpc, embassy, cfg feature flags

---

### Task 1: Move sim handlers + `postcard_sim_server_task` from `postcard.rs` to `simulation.rs`

**Files:**
- Modify: `code/flight-computer/src/tasks/postcard.rs` — lines 55-101 (remove `pub mod simulator`)
- Modify: `code/flight-computer/src/tasks/simulation.rs` — add imports, 4 handlers, `postcard_sim_server_task`

- [ ] **Step 1: Remove `pub mod simulator` from `postcard.rs`**

Delete lines 55 through 101 in `postcard.rs` (the entire `#[cfg(feature = "impl_sim")] pub mod simulator { ... }` block).

Currently:
```rust
#[cfg(feature = "impl_sim")]
pub mod simulator {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    // ... 4 handler functions + postcard_sim_server_task (lines 56-101)
}
```

Remove all of it. Nothing replaces it — the module is deleted.

- [ ] **Step 2: Add imports to `simulation.rs`**

Add these imports to `simulation.rs` (merge with existing imports):

```rust
use core::sync::atomic::{AtomicU32, Ordering};

use crate::{
    config::PostcardConfig,
    interfaces::Led,
    interfaces::impls::simulation::{
        arming_system::SimArming,
        sensor::{SimAltimeter, SimGps, SimImu},
    },
    log::{error, warn},
    sync::FLIGHT_STATE_WATCH,
};
use postcard_rpc::header::{VarHeader, VarSeq};
use postcard_rpc::server::{Sender, WireTx};
use proto::{
    actuator_data::ActuatorStatus,
    sensor_data::{AltimeterData, GpsData, ImuData},
    RecordData, SimFlightStateTopic,
};
use super::postcard::Context;
```

- [ ] **Step 3: Add the 4 sim handler functions to `simulation.rs`**

Place these right after the imports, before the existing `start_pil_flight_computer`:

```rust
pub fn sim_altimeter_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: AltimeterData, _out: &Sender<Tx>) {
    SimAltimeter::update_data(data);
}

pub fn sim_gps_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: GpsData, _out: &Sender<Tx>) {
    SimGps::update_data(data);
}

pub fn sim_imu_update<Tx: WireTx>(_context: &mut Context, _header: VarHeader, data: ImuData, _out: &Sender<Tx>) {
    SimImu::update_data(data);
}

pub fn sim_arming_activate<Tx: WireTx>(_context: &mut Context, _header: VarHeader, _data: ActuatorStatus, _out: &Sender<Tx>) {
    SimArming::activate();
}
```

- [ ] **Step 4: Add `postcard_sim_server_task` to `simulation.rs`**

Place this after the handlers:

```rust
/// Handles the server management for the fc-sim socket.
/// Panics on any disconnect — FC ↔ simulator desync is unrecoverable.
pub async fn postcard_sim_server_task<Tx, Rx, Buf, D, LED>(
    mut server: Server<Tx, Rx, Buf, D>,
    mut led: LED,
) -> !
where
    Tx: postcard_rpc::server::WireTx,
    Rx: postcard_rpc::server::WireRx,
    Buf: DerefMut<Target = [u8]>,
    D: postcard_rpc::server::Dispatch<Tx = Tx>,
    LED: Led,
{
    led.on().await.unwrap_or_else(|e| warn!("Postcard sim server: Status Led error: {:?}", e));
    // `ServerError` may lack Debug/Display in no_std context; just log the exit.
    let _ = server.run().await;
    error!("sim server: run exited (connection dropped)");
    led.off().await.unwrap_or_else(|e| warn!("Postcard sim server: Status Led error: {:?}", e));
    panic!("fc-sim connection closed: FC and simulator desynced");
}
```

- [ ] **Step 5: Fix internal `use` in `simulation.rs`**

Remove these imports from the existing `use crate::tasks::{...}` line since they're now defined in the same file:
- `flight_state_sim_publisher_task` (will define in Task 2)
- `postcard_sim_server_task` (just added)

The existing long `use crate::tasks::{...}` should become:
```rust
use crate::tasks::{
    finite_state_machine_task, groundstation_task, postcard_server_task,
    run_flight_computer, sensor_task, storage_task,
};
```

---

### Task 2: Merge `flight_state_sim_publisher.rs` into `simulation.rs`

**Files:**
- Delete: `code/flight-computer/src/tasks/flight_state_sim_publisher.rs`
- Modify: `code/flight-computer/src/tasks/simulation.rs` — add `flight_state_sim_publisher_task` function

- [ ] **Step 1: Add `flight_state_sim_publisher_task` to `simulation.rs`**

Place this function after `postcard_sim_server_task`:

```rust
static UID_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Publishes `FlightState` changes on `SimFlightStateTopic` for the simulator.
#[inline]
pub async fn flight_state_sim_publisher_task<Tx>(postcard_sender: &PostcardSender<Tx>)
where
    Tx: WireTx,
{
    let mut flight_state_receiver = FLIGHT_STATE_WATCH
        .receiver()
        .expect("Not enough flight state consumers");
    loop {
        let record = flight_state_receiver.changed().await;
        if let RecordData::FlightState(state) = record.payload().clone() {
            if postcard_sender
                .publish::<SimFlightStateTopic>(
                    VarSeq::Seq4(UID_COUNTER.fetch_add(1, Ordering::Relaxed)),
                    &state,
                )
                .await
                .is_err()
            {
                warn!("flight_state_sim_publisher: publish failed");
            }
        }
    }
}
```

Note: the imports for this (`AtomicU32`, `Ordering`, `FLIGHT_STATE_WATCH`, `VarSeq`, `PostcardSender`, `RecordData`, `SimFlightStateTopic`) were already added in Task 1 Step 2.

- [ ] **Step 2: Delete `flight_state_sim_publisher.rs`**

Run:
```bash
Remove-Item -LiteralPath "code/flight-computer/src/tasks/flight_state_sim_publisher.rs"
```

---

### Task 3: Update `mod.rs` re-exports

**Files:**
- Modify: `code/flight-computer/src/tasks/mod.rs`

- [ ] **Step 1: Change `postcard_sim_server_task` re-export source**

In `mod.rs`, change:
```rust
#[cfg(feature = "impl_sim")]
pub use postcard::simulator::postcard_sim_server_task;
```
to:
```rust
#[cfg(feature = "impl_sim")]
pub use simulation::postcard_sim_server_task;
```

- [ ] **Step 2: Remove `flight_state_sim_publisher` module and re-export**

Remove these two lines from `mod.rs`:
```rust
#[cfg(feature = "impl_sim")]
mod flight_state_sim_publisher;
#[cfg(feature = "impl_sim")]
pub use flight_state_sim_publisher::flight_state_sim_publisher_task;
```

Note: `run_flight_computer` references `flight_state_sim_publisher_task` only as a parameter name, not a module-level identifier. No change needed there.

---

### Task 4: Update external import paths

**Files:**
- Modify: `code/flight-computer-host/src/dispatch.rs`
- Modify: `code/ground-station-backend/src/bin/sitl/postcard_server.rs`

- [ ] **Step 1: Fix `dispatch.rs` import path**

In `dispatch.rs` lines 4-6, change:
```rust
use flight_computer::tasks::postcard::simulator::{
    sim_altimeter_update, sim_arming_activate, sim_gps_update, sim_imu_update,
};
```
to:
```rust
use flight_computer::tasks::simulation::{
    sim_altimeter_update, sim_arming_activate, sim_gps_update, sim_imu_update,
};
```

- [ ] **Step 2: Fix `sitl/postcard_server.rs` import path**

In `postcard_server.rs` line 1, change:
```rust
use flight_computer::tasks::postcard::{Context, embassy_time_tick_hz_handler, ping_handler, simulator::{sim_altimeter_update, sim_arming_activate, sim_gps_update, sim_imu_update}};
```
to:
```rust
use flight_computer::tasks::postcard::{Context, embassy_time_tick_hz_handler, ping_handler};
use flight_computer::tasks::simulation::{sim_altimeter_update, sim_arming_activate, sim_gps_update, sim_imu_update};
```

---

### Task 5: Build verification

- [ ] **Step 1: Run cargo check**

Run from `code/`:
```bash
cargo check --workspace
```

Expected: clean build with no errors.

- [ ] **Step 2: Run clippy**

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: clean with no new warnings.
