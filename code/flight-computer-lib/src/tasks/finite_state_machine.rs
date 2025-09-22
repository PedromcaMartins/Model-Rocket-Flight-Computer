use defmt_or_log::info;
use embassy_sync::{blocking_mutex::raw::RawMutex, watch::Sender, signal::Signal};
use switch_hal::WaitSwitch;
use telemetry_messages::{Altitude, FlightState};

use crate::{config::{ApogeeDetectorConfig, TouchdownDetectorConfig}, interfaces::DeploymentSystem, core::state_machine::FlightStateMachine};

#[inline]
pub async fn finite_state_machine_task<
    WS, D, M,
    const CONSUMERS: usize,
> (
    arm_button: WS,
    deployment_system: D,
    latest_altitude_signal: &'static Signal<M, Altitude>,

    apogee_detector_config: ApogeeDetectorConfig,
    touchdown_detector_config: TouchdownDetectorConfig,

    flight_state_sender: Sender<'static, M, FlightState, CONSUMERS>,
)
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static,
{
    let fsm = FlightStateMachine::new(
        arm_button, 
        deployment_system, 
        latest_altitude_signal,

        apogee_detector_config,
        touchdown_detector_config,
    );
    flight_state_sender.send(FlightState::PreArmed);
    info!("Flight Computer Pre-Armed");

    let fsm = fsm.wait_arm().await;
    flight_state_sender.send(FlightState::Armed);
    info!("Flight Computer Armed");

    let fsm = fsm.wait_activate_recovery().await;
    flight_state_sender.send(FlightState::RecoveryActivated);
    info!("Recovery System Activated");

    let _ = fsm.wait_touchdown().await;
    flight_state_sender.send(FlightState::Touchdown);
    info!("Touchdown Detected");
}
