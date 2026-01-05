use core::marker::PhantomData;

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::Timer;
use switch_hal::WaitSwitch;
use proto::Altitude;
use proto::uom::si::length::meter;
use defmt_or_log::{error, info};

use crate::{config::{ApogeeDetectorConfig, TouchdownDetectorConfig}, core::state_machine::{states::{Armed, PreArmed}, FlightStateMachine}, interfaces::DeploymentSystem};

impl<WS, D, M> FlightStateMachine<WS, D, M, PreArmed>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static
{
    pub const fn new(
        arm_button: WS,
        deployment_system: D,
        latest_altitude_signal: &'static Signal<M, Altitude>,

        apogee_detector_config: ApogeeDetectorConfig,
        touchdown_detector_config: TouchdownDetectorConfig,
    ) -> Self {
        Self {
            arm_button,
            deployment_system,
            latest_altitude_signal,
            phantom_data: PhantomData,

            apogee_detector_config,
            touchdown_detector_config,

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

    pub async fn wait_arm(mut self) -> FlightStateMachine<WS, D, M, Armed> {
        self.await_arm_button().await;

        let launchpad_altitude = self.latest_altitude_signal.wait().await;
        info!("Launchpad Altitude: {} m", launchpad_altitude.get::<meter>());

        self.transition_to(Some(launchpad_altitude))
    }
}
