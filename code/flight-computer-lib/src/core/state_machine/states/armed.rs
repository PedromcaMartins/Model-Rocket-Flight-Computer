use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_time::Timer;
use switch_hal::WaitSwitch;
use uom::si::length::meter;
use defmt_or_log::{error, info, Debug2Format};

use crate::{core::state_machine::{detectors::ApogeeDetector, states::{Armed, RecoveryActivated}, FlightStateMachine}, interfaces::DeploymentSystem};

impl<WS, D, M> FlightStateMachine<WS, D, M, Armed>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static
{
    async fn await_deployment_system(&mut self) {
        loop {
            match self.deployment_system.deploy() {
                Ok(()) => {
                    info!("Deployment system activated");
                    return;
                },
                Err(e) => {
                    error!("Deployment system activation failed: {:?}", Debug2Format(&e));
                    Timer::after_secs(1).await;
                }
            }
        }
    }

    pub async fn wait_activate_recovery(mut self) -> FlightStateMachine<WS, D, M, RecoveryActivated> {
        let altitude_above_launchpad = ApogeeDetector::new(
            self.latest_altitude_signal,
            self.launchpad_altitude.expect("Launchpad altitude should have been set in Armed state"),
            self.apogee_detector_config,
        ).await
        .await_apogee()
        .await;

        info!("Apogee of {} m Reached!", altitude_above_launchpad.get::<meter>());

        self.await_deployment_system().await;

        self.transition_to(None)
    }
}
