use embassy_time::Timer;
use proto::uom::si::length::meter;
use defmt_or_log::{error, info, Debug2Format};

use crate::{core::state_machine::{FlightStateMachine, detectors::ApogeeDetector, states::{Armed, RecoveryActivated}}, interfaces::{ArmingSystem, DeploymentSystem, Led}};

impl<A, LedA, D, LedD> FlightStateMachine<A, LedA, D, LedD, Armed>
where
    A: ArmingSystem,
    LedA: Led,
    D: DeploymentSystem,
    LedD: Led,
{
    async fn await_deployment_system(&mut self) {
        loop {
            match self.deployment_system.deploy().await {
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

    pub async fn wait_activate_recovery(mut self) -> FlightStateMachine<A, LedA, D, LedD, RecoveryActivated> {
        let altitude_above_launchpad = ApogeeDetector::new(
            self.launchpad_altitude.expect("Launchpad altitude should have been set in Armed state"),
        ).await
        .await_apogee()
        .await;

        info!("Apogee of {} m Reached!", altitude_above_launchpad.get::<meter>());

        self.await_deployment_system().await;
        self.deployment_system_led.on().await.ok();

        self.transition()
    }
}
