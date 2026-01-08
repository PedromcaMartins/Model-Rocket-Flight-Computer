use core::marker::PhantomData;

use embassy_time::Timer;
use proto::uom::si::length::meter;
use defmt_or_log::{error, info};

use crate::{core::state_machine::{FlightStateMachine, states::{Armed, PreArmed}}, interfaces::{ArmingSystem, DeploymentSystem}, sync::LATEST_ALTITUDE_SIGNAL};

impl<A, D> FlightStateMachine<A, D, PreArmed>
where
    A: ArmingSystem,
    D: DeploymentSystem,
{
    pub const fn new(
        arm_button: A,
        deployment_system: D,
    ) -> Self {
        Self {
            arm_button,
            deployment_system,
            phantom_data: PhantomData,

            launchpad_altitude: None,
        }
    }

    pub async fn wait_arm(mut self) -> FlightStateMachine<A, D, Armed> {
        loop {
            if self.arm_button.wait_arm().await.is_ok() {
                info!("Arm button pressed");
                break;
            }
            error!("Failed to wait for button press");
            Timer::after_secs(1).await;
        }

        let launchpad_altitude = LATEST_ALTITUDE_SIGNAL.wait().await;
        info!("Launchpad Altitude: {} m", launchpad_altitude.get::<meter>());
        self.launchpad_altitude = Some(launchpad_altitude);

        self.transition()
    }
}
