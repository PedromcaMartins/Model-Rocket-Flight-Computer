use std::sync::Arc;

use chrono::{Local, Timelike};
use embassy_time::Instant;
use telemetry_messages::{nalgebra::Quaternion, nmea::sentences::FixType, Acceleration, AltimeterMessage, Altitude, Angle, AngularVelocity, EulerAngles, FixTypeWrapper, GpsCoordinates, GpsMessage, ImuMessage, MagneticFluxDensity, Pressure, ThermodynamicTemperature, Time, Timestamp, Vector3, Velocity};
use tokio::{sync::{mpsc, watch, Mutex}, time::sleep};
use uom::si::{acceleration::meter_per_second_squared, angle::degree, angular_velocity::radian_per_second, length::meter, magnetic_flux_density::tesla, pressure::pascal, thermodynamic_temperature::degree_celsius, time::{microsecond, millisecond}, velocity::meter_per_second};

pub struct SimulatorConfig {
    tick_duration: Time,
    initial_altitude: Altitude,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            tick_duration: Time::new::<millisecond>(10.),
            initial_altitude: Altitude::new::<meter>(20.0),
        }
    }
}

#[derive(Clone)]
pub struct Simulator(Arc<Mutex<SimulatorInner>>);

struct SimulatorInner {
    button_tx: watch::Sender<bool>,
    deployment_rx: watch::Receiver<bool>,
    alt_tx: mpsc::Sender<AltimeterMessage>,
    gps_tx: mpsc::Sender<GpsMessage>,
    imu_tx: mpsc::Sender<ImuMessage>,
    sd_card_detect_tx: watch::Sender<bool>,
    sd_card_status_led_rx: watch::Receiver<bool>,

    // simulator config
    config: SimulatorConfig,
    // // physics config
    // physics: PhysicsConfig,
    // // sensor fault/latency config
    // faults: FaultConfig,

    altitude: Altitude,
    velocity: Velocity,
    acceleration: Acceleration,

    // armed: bool,
    // ignited: bool,
    // parachute_deployed: bool,

    // t_start: Instant,
    // t_ignite: Option<Instant>,
    // t_deploy: Option<Instant>,
}

impl Simulator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        button_tx: watch::Sender<bool>,
        deployment_rx: watch::Receiver<bool>,
        alt_tx: mpsc::Sender<AltimeterMessage>,
        gps_tx: mpsc::Sender<GpsMessage>,
        imu_tx: mpsc::Sender<ImuMessage>,
        sd_card_detect_tx: watch::Sender<bool>,
        sd_card_status_led_rx: watch::Receiver<bool>,
        config: SimulatorConfig,
    ) -> Self {
        Self(Arc::new(Mutex::new(
            SimulatorInner {
                button_tx,
                deployment_rx,
                alt_tx,
                gps_tx,
                imu_tx,
                sd_card_detect_tx,
                sd_card_status_led_rx,

                altitude: config.initial_altitude,
                velocity: Velocity::new::<meter_per_second>(0.0),
                acceleration: Acceleration::new::<meter_per_second_squared>(0.0),

                config,
            }
        )))
    }

    pub fn start(self) {
        let sim = self.clone();

        // spawn sensor physics loop
        tokio::spawn(sim.physics_and_sensor_loop());
    }

    async fn physics_and_sensor_loop(self) {
        let inner = self.0.lock().await;
        let dt = inner.config.tick_duration;
        drop(inner);
        
        loop {
            let inner = self.0.lock().await;

            drop(inner);

            // send sensor messages
            self.send_altimeter_message().await;
            self.send_gps_message().await;
            self.send_imu_message().await;

            sleep(tokio::time::Duration::from_micros(dt.get::<microsecond>() as u64)).await;
        }
    }

    async fn send_altimeter_message(&self) {
        let inner = self.0.lock().await;
        let msg = AltimeterMessage { // TODO
            altitude: inner.altitude,
            pressure: Pressure::new::<pascal>(101325.0), // sea level standard
            temperature: ThermodynamicTemperature::new::<degree_celsius>(20.0),
            timestamp: Instant::now().as_micros(),
        };
        inner.alt_tx.send(msg).await.expect("Failed to send altimeter message: Altimeter data will not be updated, check if the receiver is still active");
    }

    async fn send_gps_message(&self) {
        let ts = Local::now();
        let fix_time = Timestamp { hour: ts.hour() as u8, minute: ts.minute() as u8, second: ts.second() as u8 };

        let inner = self.0.lock().await;
        let msg = GpsMessage { // TODO
            fix_time,
            fix_type: FixTypeWrapper::new(FixType::Simulation),
            coordinates: GpsCoordinates {
                latitude: 37.7749,   // San Francisco, CA
                longitude: -122.4194,
            },
            altitude: inner.altitude,
            num_of_fix_satellites: 12,
            timestamp: Instant::now().as_micros(),
        };
        inner.gps_tx.send(msg).await.expect("Failed to send GPS message: GPS data will not be updated, check if the receiver is still active");
    }

    async fn send_imu_message(&self) {
        let angle = Angle::new::<degree>(0.0);
        let gyro = AngularVelocity::new::<radian_per_second>(0.0);
        let mag = MagneticFluxDensity::new::<tesla>(0.0);

        let inner = self.0.lock().await;
        let accel = inner.acceleration;
        let msg = ImuMessage { // TODO
            euler_angles: EulerAngles { roll: angle, pitch: angle, yaw: angle },
            quaternion: Quaternion::identity(),
            linear_acceleration: Vector3::new(accel, accel, accel),
            gravity: Vector3::new(accel, accel, accel),
            acceleration: Vector3::new(accel, accel, accel),
            gyro: Vector3::new(gyro, gyro, gyro),
            mag: Vector3::new(mag, mag, mag),
            temperature: ThermodynamicTemperature::new::<degree_celsius>(20.0),
            timestamp: Instant::now().as_micros(),
        };
        inner.imu_tx.send(msg).await.expect("Failed to send IMU message: IMU data will not be updated, check if the receiver is still active");
    }
}
