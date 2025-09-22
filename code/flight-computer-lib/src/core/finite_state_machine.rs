use core::marker::PhantomData;

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::Timer;
use switch_hal::WaitSwitch;
use telemetry_messages::Altitude;
use uom::si::length::meter;
use defmt_or_log::{error, info, Debug2Format};

use crate::{config::{ApogeeDetectorConfig, TouchdownDetectorConfig}, interfaces::DeploymentSystem, core::finite_state_machine::{apogee_detector::ApogeeDetector, touchdown_detector::TouchdownDetector}};

mod apogee_detector;
mod touchdown_detector;

pub struct PreArmed;
pub struct Armed;
pub struct RecoveryActivated;
pub struct Touchdown;

pub struct FiniteStateMachine<WS, D, M, S>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static,
    S: FlightState,
{
    arm_button: WS,
    deployment_system: D,
    latest_altitude_signal: &'static Signal<M, Altitude>,
    phantom_data: PhantomData<S>,

    apogee_detector_config: ApogeeDetectorConfig,
    touchdown_detector_config: TouchdownDetectorConfig,

    launchpad_altitude: Option<Altitude>,
}

pub trait FlightState {}
impl FlightState for PreArmed {}
impl FlightState for Armed {}
impl FlightState for RecoveryActivated {}
impl FlightState for Touchdown {}

impl<WS, D, M> FiniteStateMachine<WS, D, M, PreArmed>
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

    async fn wait_for_arm_button(&mut self) {
        loop {
            if self.arm_button.wait_active().await.is_ok() {
                info!("Arm button pressed");
                return;
            }
            error!("Failed to wait for button press");
            Timer::after_secs(1).await;
        }
    }

    pub async fn wait_arm(mut self) -> FiniteStateMachine<WS, D, M, Armed> {
        self.wait_for_arm_button().await;
        info!("Flight Computer Armed");

        let launchpad_altitude = self.latest_altitude_signal.wait().await;
        info!("Launchpad Altitude: {} m", launchpad_altitude.get::<meter>());

        FiniteStateMachine {
            arm_button: self.arm_button,
            deployment_system: self.deployment_system,
            latest_altitude_signal: self.latest_altitude_signal,
            phantom_data: PhantomData,

            apogee_detector_config: self.apogee_detector_config,
            touchdown_detector_config: self.touchdown_detector_config,

            launchpad_altitude: Some(launchpad_altitude),
        }
    }
}

impl<WS, D, M> FiniteStateMachine<WS, D, M, Armed>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static
{
    pub async fn wait_activate_recovery(mut self) -> FiniteStateMachine<WS, D, M, RecoveryActivated> {
        let altitude_above_launchpad = ApogeeDetector::new(
            self.latest_altitude_signal,
            self.launchpad_altitude.expect("Launchpad altitude should have been set in Armed state"),
            self.apogee_detector_config,
        ).await
        .await_apogee()
        .await;

        info!("Apogee of {} m Reached!", altitude_above_launchpad.get::<meter>());

        loop {
            match self.deployment_system.deploy() {
                Ok(()) => {
                    info!("Deployment system activated");
                    break;
                },
                Err(e) => {
                    error!("Deployment system activation failed: {:?}", Debug2Format(&e));
                    Timer::after_millis(100).await;
                }
            }
        }

        FiniteStateMachine {
            arm_button: self.arm_button,
            deployment_system: self.deployment_system,
            latest_altitude_signal: self.latest_altitude_signal,
            phantom_data: PhantomData,

            apogee_detector_config: self.apogee_detector_config,
            touchdown_detector_config: self.touchdown_detector_config,

            launchpad_altitude: self.launchpad_altitude,
        }
    }
}

impl<WS, D, M> FiniteStateMachine<WS, D, M, RecoveryActivated>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    D: DeploymentSystem,
    M: RawMutex + 'static
{
    pub async fn wait_touchdown(self) -> FiniteStateMachine<WS, D, M, Touchdown> {
        let altitude = TouchdownDetector::new(
            self.latest_altitude_signal,
            self.touchdown_detector_config,
        ).await
        .await_touchdown()
        .await;

        info!("Touchdown of {} m!", altitude.get::<meter>());

        FiniteStateMachine {
            arm_button: self.arm_button,
            deployment_system: self.deployment_system,
            latest_altitude_signal: self.latest_altitude_signal,
            phantom_data: PhantomData,

            apogee_detector_config: self.apogee_detector_config,
            touchdown_detector_config: self.touchdown_detector_config,

            launchpad_altitude: self.launchpad_altitude,
        }
    }
}
