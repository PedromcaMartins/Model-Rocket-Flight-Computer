use core::marker::PhantomData;

use embassy_time::Timer;
use switch_hal::WaitSwitch;
use proto::uom::si::length::meter;
use defmt_or_log::{error, info};

use crate::{core::state_machine::{FlightStateMachine, states::{Armed, PreArmed}}, interfaces::DeploymentSystem, sync::LATEST_ALTITUDE_SIGNAL};

impl<WS, D> FlightStateMachine<WS, D, PreArmed>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
{
    pub const fn new(
        arm_button: WS,
        deployment_system: D,
    ) -> Self {
        Self {
            arm_button,
            deployment_system,
            phantom_data: PhantomData,

            launchpad_altitude: None,
        }
    }

    async fn await_arm_button(&mut self) {
        loop {
            if self.arm_button.wait_active().await.is_ok() {
                info!("Arm button pressed");
                return;
            }
            error!("Failed to wait for button press");
            Timer::after_secs(1).await;
        }
    }

    pub async fn wait_arm(mut self) -> FlightStateMachine<WS, D, Armed> {
        self.await_arm_button().await;

        let launchpad_altitude = LATEST_ALTITUDE_SIGNAL.wait().await;
        info!("Launchpad Altitude: {} m", launchpad_altitude.get::<meter>());
        self.launchpad_altitude = Some(launchpad_altitude);

        self.transition()
    }
}
