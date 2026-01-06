use core::marker::PhantomData;

use switch_hal::WaitSwitch;
use proto::Altitude;

use crate::{core::state_machine::states::FlightState, interfaces::DeploymentSystem};

mod states;
mod detectors;

pub struct FlightStateMachine<WS, D, S>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    S: FlightState,
{
    arm_button: WS,
    deployment_system: D,
    phantom_data: PhantomData<S>,

    /// The following attributes describe 
    launchpad_altitude: Option<Altitude>,
}

impl<WS, D, S> FlightStateMachine<WS, D, S>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    S: FlightState,
{
    // Common transition helper
    fn transition<T: FlightState>(self) -> FlightStateMachine<WS, D, T> {
        FlightStateMachine {
            arm_button: self.arm_button,
            deployment_system: self.deployment_system,
            launchpad_altitude: self.launchpad_altitude,
            phantom_data: PhantomData,
        }
    }
}
