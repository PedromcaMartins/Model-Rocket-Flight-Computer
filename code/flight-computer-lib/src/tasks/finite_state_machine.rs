use defmt_or_log::info;
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use switch_hal::WaitSwitch;
use uom::si::f32::Length;

use crate::model::finite_state_machine::FiniteStateMachine;

#[inline]
pub async fn finite_state_machine_task<WS, M> (
    arm_button: WS,
    latest_altitude_signal: &'static Signal<M, Length>,
)
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    M: RawMutex + 'static,
{
    let fsm = FiniteStateMachine::new(arm_button, latest_altitude_signal);
    info!("Flight Computer Pre-Armed");

    let fsm = fsm.wait_arm().await;
    info!("Flight Computer Armed");

    let fsm = fsm.wait_activate_recovery().await;
    info!("Recovery System Activated");

    let _ = fsm.wait_touchdown().await;
    info!("Touchdown Detected");
}
