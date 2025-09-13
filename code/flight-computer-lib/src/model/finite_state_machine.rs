use core::marker::PhantomData;

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use embassy_time::Timer;
use switch_hal::WaitSwitch;
use telemetry_messages::Altitude;
use uom::si::length::meter;
use defmt_or_log::{error, info};

pub struct PreArmed;
pub struct Armed;
pub struct RecoveryActivated;
pub struct Touchdown;

pub struct FiniteStateMachine<WS, M, S>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    M: RawMutex + 'static,
    S: FlightState,
{
    arm_button: WS,
    latest_altitude_signal: &'static Signal<M, Altitude>,
    phantom_data: PhantomData<S>,

    launchpad_altitude: Option<Altitude>,
    recovery_activated_altitude: Option<Altitude>,
    #[allow(dead_code)]
    touchdown_altitude: Option<Altitude>,
}

pub trait FlightState {}
impl FlightState for PreArmed {}
impl FlightState for Armed {}
impl FlightState for RecoveryActivated {}
impl FlightState for Touchdown {}

impl<WS, M> FiniteStateMachine<WS, M, PreArmed>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    M: RawMutex + 'static
{
    pub const fn new(
        arm_button: WS,
        latest_altitude_signal: &'static Signal<M, Altitude>,
    ) -> Self {
        Self {
            latest_altitude_signal,
            arm_button,
            phantom_data: PhantomData,

            launchpad_altitude: None,
            recovery_activated_altitude: None,
            touchdown_altitude: None,
        }
    }

    async fn wait_for_arm_button(&mut self) {
        loop {
            if self.arm_button.wait_active().await.is_ok() {
                info!("Arm button pressed");
                return;
            }
            error!("Arm button: Failed to wait for button press");
            Timer::after_secs(1).await;
        }
    }

    pub async fn wait_arm(mut self) -> FiniteStateMachine<WS, M, Armed> {
        self.wait_for_arm_button().await;
        info!("Flight Computer Armed");

        let launchpad_altitude = self.latest_altitude_signal.wait().await;
        info!("Launchpad Altitude: {} m", launchpad_altitude.get::<meter>());

        FiniteStateMachine {
            latest_altitude_signal: self.latest_altitude_signal,
            arm_button: self.arm_button,
            phantom_data: PhantomData,

            launchpad_altitude: Some(launchpad_altitude),
            recovery_activated_altitude: None,
            touchdown_altitude: None,
        }
    }
}

impl<WS, M> FiniteStateMachine<WS, M, Armed>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    M: RawMutex + 'static
{
    pub async fn wait_activate_recovery(self) -> FiniteStateMachine<WS, M, RecoveryActivated> {
        loop {
            let altitude = self.latest_altitude_signal.wait().await;
            let launchpad_altitude = self.launchpad_altitude.expect("Launchpad altitude should be set in Armed state");
            let altitude_above_launchpad = altitude - launchpad_altitude;

            let min_altitude_deployment = Altitude::new::<meter>(2.0);

            if altitude_above_launchpad > min_altitude_deployment {
                info!("Apogee of {} m Reached!", altitude_above_launchpad.get::<meter>());
                info!("Recovery Activated");

                return FiniteStateMachine {
                    latest_altitude_signal: self.latest_altitude_signal,
                    arm_button: self.arm_button,
                    phantom_data: PhantomData,

                    launchpad_altitude: self.launchpad_altitude,
                    recovery_activated_altitude: Some(altitude),
                    touchdown_altitude: None,
                }
            }
        }
    }
}

impl<WS, M> FiniteStateMachine<WS, M, RecoveryActivated>
where
    WS: WaitSwitch + 'static,
    <WS as WaitSwitch>::Error: core::fmt::Debug,
    M: RawMutex + 'static
{
    pub async fn wait_touchdown(self) -> FiniteStateMachine<WS, M, Touchdown> {
        use uom::si::length::meter;

        loop {
            let altitude = self.latest_altitude_signal.wait().await;
            let launchpad_altitude = self.launchpad_altitude.expect("Launchpad altitude should be set in Armed state");
            let altitude_above_launchpad = altitude - launchpad_altitude;

            let max_altitude_touchdown = Altitude::new::<meter>(2.0);

            if altitude_above_launchpad <= max_altitude_touchdown {
                info!("Touchdown!");

                return FiniteStateMachine {
                    latest_altitude_signal: self.latest_altitude_signal,
                    arm_button: self.arm_button,
                    phantom_data: PhantomData,

                    launchpad_altitude: self.launchpad_altitude,
                    recovery_activated_altitude: self.recovery_activated_altitude,
                    touchdown_altitude: Some(altitude),
                }
            }
        }
    }
}
