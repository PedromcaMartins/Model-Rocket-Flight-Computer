use std::sync::Arc;

use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};
use tokio::{select, sync::{Mutex, mpsc, watch}, time::{Duration, Instant, sleep, sleep_until}};
use uom::si::time::second;

use crate::simulator::{physics::{config::PhysicsConfig, PhysicsSimulator}, scripted_events::ScriptedEvents};

pub mod physics;
pub mod scripted_events;

#[derive(Clone, Default)]
pub struct SimulatorConfig {
    physics: PhysicsConfig,
    scripted_events: ScriptedEvents,
}

pub struct Simulator {
    // physics simulator
    physics: Arc<Mutex<PhysicsSimulator>>,

    // this is sent to physics simulator (actuator)
    deployment_rx: watch::Receiver<bool>,

    // this is received from physics simulator (sensors)
    alt_tx: mpsc::Sender<AltimeterMessage>,
    gps_tx: mpsc::Sender<GpsMessage>,
    imu_tx: mpsc::Sender<ImuMessage>,

    // scripted_events
    arm_button_tx: watch::Sender<bool>,

    config: SimulatorConfig,
}

impl Simulator {
    pub fn new(
        deployment_rx: watch::Receiver<bool>,
        alt_tx: mpsc::Sender<AltimeterMessage>,
        gps_tx: mpsc::Sender<GpsMessage>,
        imu_tx: mpsc::Sender<ImuMessage>,
        arm_button_tx: watch::Sender<bool>,
        config: SimulatorConfig,
    ) -> Self {
        Self{
            physics: Arc::new(Mutex::new(
                PhysicsSimulator::new(config.physics)
            )),
            deployment_rx,
            alt_tx,
            gps_tx,
            imu_tx,
            arm_button_tx,
            config,
        }
    }

    pub async fn run(self) {
        tokio::spawn(Self::scripted_events(
            self.config.scripted_events, 
            self.arm_button_tx,
            self.physics.clone()
        ));

        tokio::spawn(Self::actuator_loop(
            self.deployment_rx, 
            self.physics.clone()
        ));

        select! {
            _ = tokio::spawn(Self::physics_loop(
                self.config.physics, 
                self.physics.clone()
            )) => tracing::error!("Physics loop ended unexpectedly"),
            _ = tokio::spawn(Self::sensor_loop(
                self.alt_tx, 
                self.gps_tx, 
                self.imu_tx, 
                self.physics.clone()
            )) => tracing::error!("Sensor loop ended unexpectedly"),
        }
    }

    // pub async fn ignite_engine(&self) {
    //     let mut physics = self.physics.lock().await;
    //     physics.ignite_engine();
    // }

    async fn physics_loop(
        config: PhysicsConfig, 
        physics: Arc<Mutex<PhysicsSimulator>>
    ) {
        let time_step = config.time_step_period.get::<second>();
        let mut interval = tokio::time::interval(Duration::from_secs_f32(time_step * config.time_acceleration_factor));

        loop {
            interval.tick().await;

            let mut physics = physics.lock().await;
            physics.advance_simulation();
        }
    }

    async fn sensor_loop(
        alt_tx: mpsc::Sender<AltimeterMessage>,
        gps_tx: mpsc::Sender<GpsMessage>,
        imu_tx: mpsc::Sender<ImuMessage>,
        physics: Arc<Mutex<PhysicsSimulator>>,
    ) {
        loop {
            select! {
                Ok(permit) = alt_tx.reserve() => {
                    let physics = physics.lock().await;
                    let state = physics.current_state();
                    permit.send(state.into());
                    tracing::debug!("Altimeter message sent");
                },
                Ok(permit) = gps_tx.reserve() => {
                    let physics = physics.lock().await;
                    let state = physics.current_state();
                    permit.send(state.into());
                    tracing::debug!("GPS message sent");
                },
                Ok(permit) = imu_tx.reserve() => {
                    let physics = physics.lock().await;
                    let state = physics.current_state();
                    permit.send(state.into());
                    tracing::debug!("IMU message sent");
                },
            }
        }
    }

    async fn actuator_loop(
        mut deployment_rx: watch::Receiver<bool>,
        physics: Arc<Mutex<PhysicsSimulator>>,
    ) {
        loop {
            select! {
                res = deployment_rx.changed() => {
                    if res.is_err() {
                        tracing::error!("Parachute deployment channel closed");
                        return;
                    }

                    let deployed = *deployment_rx.borrow_and_update();
                    tracing::info!("Parachute deployment event triggered: {}", if deployed { "deployed" } else { "not deployed" });

                    if deployed {
                        let mut physics = physics.lock().await;
                        physics.deploy_recovery();
                    }
                },
            }
        }
    }

    async fn scripted_events(
        scripted_events: ScriptedEvents,
        arm_button_tx: watch::Sender<bool>,
        physics: Arc<Mutex<PhysicsSimulator>>,
    ) {
        let start_time = Instant::now();

        // Auto arm button press
        if let Some(press_time) = scripted_events.arm_button_press_time {
            tracing::debug!("Waiting {:.3} seconds to auto press arm button", press_time.as_secs_f32());
            sleep_until(start_time + press_time).await;

            tracing::info!("Auto arm button press event triggered");
            arm_button_tx.send(true).expect("Failed to send arm button press signal");
            // Simulate button release after short delay
            sleep(Duration::from_secs(1)).await;
            arm_button_tx.send(false).expect("Failed to send arm button release signal");
        }

        // Auto ignition
        if let Some(ignition_time) = scripted_events.auto_motor_ignition {
            tracing::debug!("Waiting {:.3} seconds to auto ignite motor", ignition_time.as_secs_f32());
            sleep_until(start_time + ignition_time).await;

            tracing::info!("Auto motor ignition event triggered");
            let mut physics = physics.lock().await;
            physics.ignite_engine();
        }
    }
}
