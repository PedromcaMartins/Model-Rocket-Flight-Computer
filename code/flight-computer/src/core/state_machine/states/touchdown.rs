use defmt_or_log::info;
use embassy_sync::blocking_mutex::raw::RawMutex;
use switch_hal::WaitSwitch;

use crate::{core::state_machine::{states::Touchdown, FlightStateMachine}, interfaces::DeploymentSystem};

impl<WS, D, M> FlightStateMachine<WS, D, M, Touchdown>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static
{
    pub async fn shutdown(self) {
        info!("Shutting down flight computer.");
    }
}
