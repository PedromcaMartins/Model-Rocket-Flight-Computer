use core::future::Future;

use crate::log::info;
use embassy_futures::join::{join, join5};
use embassy_futures::select::{Either, select};

mod finite_state_machine;
pub use finite_state_machine::finite_state_machine_task;
mod sensor;
pub use sensor::sensor_task;
mod storage;
pub use storage::storage_task;
mod groundstation;
pub use groundstation::groundstation_task;
pub mod postcard;
pub use postcard::postcard_server_task;

#[cfg(feature = "impl_sim")]
pub mod simulation;

#[inline]
pub async fn run_flight_computer(
    finite_state_machine_task: impl Future<Output = ()>,
    storage_task: impl Future<Output = ()>,
    postcard_task: impl Future,
    altimeter_task: impl Future,
    gps_task: impl Future,
    imu_task: impl Future,
    groundstation_task: impl Future,
) {
    if matches!(select(
        join(finite_state_machine_task, storage_task),
        join5(postcard_task, altimeter_task, gps_task, imu_task, groundstation_task),
    ).await, Either::First(((), ()))) {
        info!("Flight Computer: Finite state machine and storage tasks completed");
    }

    info!("Flight Computer: Shutdown complete");
}