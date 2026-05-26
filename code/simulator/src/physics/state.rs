use chrono::Local;
use proto::
    sensor_data::{
        Acceleration, AltimeterData, Altitude, AngularVelocity, GpsCoordinates, GpsData, ImuData,
        MagneticFluxDensity, Time, Velocity, Vector3,
        nmea::sentences::FixType,
    }
;

use crate::config::SimulatorConfig;
use crate::types::ForceEvent;

#[derive(Debug, Clone)]
pub struct PhysicsState {
    pub time: Time,
    pub altitude: Altitude,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
    pub coordinates: GpsCoordinates,

    pub motor_ignited: Option<Time>,
    pub recovery_deployed: Option<Time>,
    pub touched_down: Option<Time>,
}

impl PhysicsState {
    pub fn is_motor_burning(&self) -> bool {
        self.motor_ignited
            .is_some_and(|t| self.time - t < SimulatorConfig::motor_burn_time())
    }

    pub fn is_flying(&self) -> bool {
        self.motor_ignited.is_some() && !self.has_touched_down()
    }

    pub fn has_touched_down(&self) -> bool {
        self.touched_down.is_some()
    }

    pub fn active_force_events(&self) -> Vec<ForceEvent> {
        if self.has_touched_down() {
            vec![ForceEvent::Gravity, ForceEvent::Ground]
        } else if self.is_flying() {
            let mut events = vec![ForceEvent::Gravity];
            if self.is_motor_burning() {
                events.push(ForceEvent::MotorThrust);
            }
            if self.recovery_deployed.is_some() {
                events.push(ForceEvent::Recovery);
            }
            events
        } else {
            vec![]
        }
    }
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            altitude: SimulatorConfig::launchpad_altitude(),
            coordinates: SimulatorConfig::LAUNCHPAD_COORDINATES,
            time: Time::default(),
            velocity: Velocity::default(),
            acceleration: Acceleration::default(),
            motor_ignited: None,
            recovery_deployed: None,
            touched_down: None,
        }
    }
}

impl From<PhysicsState> for AltimeterData {
    fn from(value: PhysicsState) -> Self {
        AltimeterData {
            altitude: value.altitude,
            pressure: SimulatorConfig::sea_level_pressure(),
            temperature: SimulatorConfig::ambient_temperature(),
        }
    }
}

impl From<PhysicsState> for ImuData {
    fn from(value: PhysicsState) -> Self {
        let gyro = AngularVelocity::default();
        let mag = MagneticFluxDensity::default();
        let zero_accel = Acceleration::default();

        ImuData {
            acceleration: Vector3::new(zero_accel, zero_accel, value.acceleration),
            gyro: Vector3::new(gyro, gyro, gyro),
            mag: Vector3::new(mag, mag, mag),
            temperature: SimulatorConfig::ambient_temperature(),
        }
    }
}

impl From<PhysicsState> for GpsData {
    fn from(value: PhysicsState) -> Self {
        GpsData {
            fix_time: Local::now().naive_local().time().into(),
            fix_type: FixType::Simulation.into(),
            coordinates: value.coordinates,
            altitude: value.altitude,
            num_of_fix_satellites: SimulatorConfig::GPS_FIX_SATELLITES,
        }
    }
}
