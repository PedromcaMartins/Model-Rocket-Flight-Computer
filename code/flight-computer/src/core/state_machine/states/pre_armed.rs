use core::marker::PhantomData;

use embassy_futures::select::{Either, select};
use embassy_time::Ticker;
use proto::uom::si::length::meter;
use defmt_or_log::{error, info};

use crate::{config::FiniteStateMachineConfig, core::state_machine::{FlightStateMachine, states::{Armed, PreArmed}}, interfaces::{ArmingSystem, DeploymentSystem, Led}, sync::LATEST_ALTITUDE_SIGNAL};

impl<A, LedA, D, LedD> FlightStateMachine<A, LedA, D, LedD, PreArmed>
where
    A: ArmingSystem,
    LedA: Led,
    D: DeploymentSystem,
    LedD: Led,
{
    pub async fn new(
        arm_button: A,
        mut arm_button_led: LedA,
        deployment_system: D,
        mut deployment_system_led: LedD,
    ) -> Self {
        if arm_button_led.off().await.is_err() { error!("FSM: Arm Button Led error"); }
        if deployment_system_led.off().await.is_err() { error!("FSM: Deployment System Led error"); }

        Self {
            arm_button,
            arm_button_led,
            deployment_system,
            deployment_system_led,
            _state: PhantomData,

            launchpad_altitude: None,
        }
    }

    pub async fn wait_arm(mut self) -> FlightStateMachine<A, LedA, D, LedD, Armed> {
        let mut waiting_arm_ticker = Ticker::every(FiniteStateMachineConfig::WAITING_ARM_INTERVAL);

        loop {
            match select(
                self.arm_button.wait_arm(),
                waiting_arm_ticker.next(),
            ).await {
                Either::First(Ok(())) => {
                    info!("Arm button pressed");
                    self.arm_button_led.on().await.ok();
                    break;
                },
                Either::First(Err(_)) => {
                    error!("Failed to wait for button press");
                },
                Either::Second(()) => {
                    self.arm_button_led.toggle().await.ok();
                },
            }
        }

        let launchpad_altitude = LATEST_ALTITUDE_SIGNAL.wait().await;
        info!("Launchpad Altitude: {} m", launchpad_altitude.get::<meter>());
        self.launchpad_altitude = Some(launchpad_altitude);

        self.transition()
    }
}
