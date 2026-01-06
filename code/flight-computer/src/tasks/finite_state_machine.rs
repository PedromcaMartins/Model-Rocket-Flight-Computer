use defmt_or_log::info;
use switch_hal::WaitSwitch;
use proto::FlightState;

use crate::{core::state_machine::FlightStateMachine, interfaces::DeploymentSystem, sync::FLIGHT_STATE_WATCH};

#[inline]
pub async fn finite_state_machine_task<
    WS, D,
> (
    arm_button: WS,
    deployment_system: D,
)
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
{
    let flight_state_sender = FLIGHT_STATE_WATCH.sender();

    let fsm = FlightStateMachine::new(
        arm_button, 
        deployment_system, 
    );
    flight_state_sender.send(FlightState::PreArmed);
    info!("Flight Computer Pre-Armed");

    let fsm = fsm.wait_arm().await;
    flight_state_sender.send(FlightState::Armed);
    info!("Flight Computer Armed");

    let fsm = fsm.wait_activate_recovery().await;
    flight_state_sender.send(FlightState::RecoveryActivated);
    info!("Recovery System Activated");

    let fsm = fsm.wait_touchdown().await;
    flight_state_sender.send(FlightState::Touchdown);
    info!("Touchdown Detected");

    fsm.shutdown().await;
}
