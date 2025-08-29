use defmt_or_log::info;
use embassy_sync::{blocking_mutex::raw::RawMutex, watch::Sender, signal::Signal};
use uom::si::f64::Length;

use crate::model::{finite_state_machine::FiniteStateMachine, system_status::FlightState};

#[inline]
pub async fn finite_state_machine_task<
    M, 
    const CONSUMERS: usize,
> (
    arm_button_pushed_signal: &'static Signal<M, ()>,
    latest_altitude_signal: &'static Signal<M, Length>,
    flight_state_sender: Sender<'static, M, FlightState, CONSUMERS>,
)
where
    M: RawMutex + 'static,
{
    let fsm = FiniteStateMachine::new(arm_button_pushed_signal);
    flight_state_sender.send(FlightState::PreArmed);
    info!("Flight Computer Pre-Armed");

    let fsm = fsm.wait_arm(latest_altitude_signal).await;
    flight_state_sender.send(FlightState::Armed);
    info!("Flight Computer Armed");

    let fsm = fsm.wait_activate_recovery().await;
    flight_state_sender.send(FlightState::RecoveryActivated);
    info!("Recovery System Activated");

    let _ = fsm.wait_touchdown().await;
    flight_state_sender.send(FlightState::Touchdown);
    info!("Touchdown Detected");
}
