use core::marker::PhantomData;

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use switch_hal::WaitSwitch;
use proto::Altitude;

use crate::{config::{ApogeeDetectorConfig, TouchdownDetectorConfig}, core::state_machine::states::FlightState, interfaces::DeploymentSystem};

mod states;
mod detectors;

pub struct FlightStateMachine<WS, D, M, S>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static,
    S: FlightState,
{
    arm_button: WS,
    deployment_system: D,
    latest_altitude_signal: &'static Signal<M, Altitude>,
    phantom_data: PhantomData<S>,

    apogee_detector_config: ApogeeDetectorConfig,
    touchdown_detector_config: TouchdownDetectorConfig,

    launchpad_altitude: Option<Altitude>,
}

impl<WS, D, M, S> FlightStateMachine<WS, D, M, S>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static,
    S: FlightState,
{
    // Common transition helper
    fn transition_to<T: FlightState>(self, new_launchpad_altitude: Option<Altitude>) -> FlightStateMachine<WS, D, M, T> {
        FlightStateMachine {
            arm_button: self.arm_button,
            deployment_system: self.deployment_system,
            latest_altitude_signal: self.latest_altitude_signal,
            apogee_detector_config: self.apogee_detector_config,
            touchdown_detector_config: self.touchdown_detector_config,
            launchpad_altitude: new_launchpad_altitude.or(self.launchpad_altitude),
            phantom_data: PhantomData,
        }
    }
}
