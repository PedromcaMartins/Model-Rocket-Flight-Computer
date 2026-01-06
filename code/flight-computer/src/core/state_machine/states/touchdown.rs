use defmt_or_log::info;
use switch_hal::WaitSwitch;

use crate::{core::state_machine::{states::Touchdown, FlightStateMachine}, interfaces::DeploymentSystem};

impl<WS, D> FlightStateMachine<WS, D, Touchdown>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
{
    pub async fn shutdown(self) {
        info!("Shutting down flight computer.");
    }
}
