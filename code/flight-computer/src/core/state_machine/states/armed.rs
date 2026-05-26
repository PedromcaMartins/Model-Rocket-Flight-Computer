use embassy_time::with_timeout;
use proto::uom::si::length::meter;
use defmt_or_log::Debug2Format;

use crate::config::ArmedConfig;
use crate::log::{error, info};
use crate::{core::state_machine::{FlightStateMachine, detectors::ApogeeDetector, states::{Armed, RecoveryActivated}}, interfaces::{ArmingSystem, DeploymentSystem, Led}};

impl<A, LedA, D, LedD> FlightStateMachine<A, LedA, D, LedD, Armed>
where
    A: ArmingSystem,
    LedA: Led,
    D: DeploymentSystem,
    LedD: Led,
{
    async fn await_deployment_system(&mut self) {
        let mut deploy_attempt = 0u32;
        loop {
            deploy_attempt += 1;
            info!("Deploy attempt #{deploy_attempt}: calling deploy()");
            match with_timeout(ArmedConfig::DEPLOY_TIMEOUT, self.deployment_system.deploy()).await {
                Err(_) => {
                    error!("Deploy attempt #{deploy_attempt} deploy() timed out, retrying");
                },
                Ok(Err(e)) => {
                    error!("Deploy attempt #{deploy_attempt} deploy() failed: {:?}", Debug2Format(&e));
                },
                Ok(Ok(())) => {
                    info!("Deploy attempt #{deploy_attempt} deploy() Ok, calling verify()");
                    match with_timeout(ArmedConfig::VERIFY_TIMEOUT, self.deployment_system.verify_deployment()).await {
                        Err(_) => {
                            error!("Deploy attempt #{deploy_attempt} verify() timed out, retrying");
                        },
                        Ok(Err(e)) => {
                            error!("Deploy attempt #{deploy_attempt} verify() error: {:?}", Debug2Format(&e));
                        },
                        Ok(Ok(false)) => {
                            error!("Deploy attempt #{deploy_attempt} verify() false, retrying");
                        },
                        Ok(Ok(true)) => {
                            info!("Deploy attempt #{deploy_attempt} verify Ok — deployment confirmed");
                            return;
                        },
                    }
                },
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
