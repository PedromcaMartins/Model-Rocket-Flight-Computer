# Simulation Code Consolidation

## Goal

Move all simulation-specific code into a single file (`code/flight-computer/src/tasks/simulation.rs`) so that:

1. `postcard.rs` no longer has a `#[cfg(feature = "impl_sim")] pub mod simulator` submodule
2. `flight_state_sim_publisher.rs` is merged into `simulation.rs`
3. `run_flight_computer` in `mod.rs` uses `#[cfg(feature = "impl_sim")]` on the `flight_state_sim_publisher_task` parameter and `core::future::pending::<()>()` as a placeholder in non-sim builds — this is already correct and unchanged

## What moves into `simulation.rs`

| Item | Source | Notes |
|---|---|---|
| `sim_altimeter_update` | `postcard.rs::simulator` | Dispatch handler, callers update import paths |
| `sim_gps_update` | `postcard.rs::simulator` | Same |
| `sim_imu_update` | `postcard.rs::simulator` | Same |
| `sim_arming_activate` | `postcard.rs::simulator` | Same |
| `postcard_sim_server_task` | `postcard.rs::simulator` | Re-exported from `mod.rs` as `tasks::postcard_sim_server_task` |
| `flight_state_sim_publisher_task` | `flight_state_sim_publisher.rs` | Entire file merged into `simulation.rs` |

## What is removed

- `postcard.rs` lines 55–101 (`#[cfg(feature = "impl_sim")] pub mod simulator { ... }`)
- `flight_state_sim_publisher.rs` (entire file)

## Import path changes

| File | Old path | New path |
|---|---|---|
| `flight-computer-host/src/dispatch.rs` | `tasks::postcard::simulator` | `tasks::simulation` |
| `ground-station-backend/src/bin/sitl/postcard_server.rs` | `tasks::postcard::simulator` | `tasks::simulation` |

## `mod.rs` changes

- `pub use postcard::simulator::postcard_sim_server_task` → `pub use simulation::postcard_sim_server_task`
- Remove `mod flight_state_sim_publisher` + `pub use flight_state_sim_publisher::...`
- Keep `#[cfg(feature = "impl_sim")] pub mod simulation;` (unchanged)
- `run_flight_computer` signature — unchanged (already correct)

## Non-goals

- No functional changes to `run_flight_computer` or any task wiring
- No feature-flag renames
- No changes to proto, simulator crate, or ground-station-backend beyond import path updates
