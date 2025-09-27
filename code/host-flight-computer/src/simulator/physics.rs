use telemetry_messages::{Acceleration, Velocity};
use uom::si::{f32::Time, time::second};

use crate::simulator::physics::{config::PhysicsConfig, state::PhysicsState};

pub mod config;
mod state;

pub struct PhysicsSimulator {
    // physics config
    config: PhysicsConfig,
    // sensor fault/latency config
    // faults: FaultConfig,

    current_state: PhysicsState,

    // t_start: Instant,
    // t_ignite: Option<Instant>,
    // t_deploy: Option<Instant>,
}

impl PhysicsSimulator {
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            config,
            current_state: PhysicsState {
                timestamp: Time::default(),
                altitude: config.launchpad_altitude,
                velocity: Velocity::default(),
                acceleration: Acceleration::default(),
                coordinates: config.launchpad_coordinates,
                motor_ignited_ts: None,
                recovery_deployed_ts: None,
                landed: false,
            },
        }
    }

    pub fn advance_simulation(&mut self) {
        let config = &self.config;
        let dt = config.time_step_period;
        let s = &mut self.current_state;

        // advance simulation time
        s.timestamp += dt;

        // --- skip simulation if motor hasnt been ignited or rocket landed ---
        if s.motor_ignited_ts.is_none() {
            return
        } else if s.landed {
            s.altitude = config.landing_altitude;
            s.velocity = Velocity::default();
            s.acceleration = Acceleration::default();
            return
        }

        // --- acceleration model ---
        if let Some(ts) = s.motor_ignited_ts && (s.timestamp - ts) < config.motor_burn_time {
            // during powered ascent
            let motor_accel = config.motor_avg_thrust / config.mass;
            s.acceleration = motor_accel - config.gravity;
        } else {
            // ballistic flight
            s.acceleration = -config.gravity;
        }

        // --- velocity integration ---
        if let Some(ts) = s.recovery_deployed_ts {
            // simple drag model toward terminal velocity
            let desired_v = -config.recovery_terminal_velocity;
            let dv = (desired_v - s.velocity) * ((s.timestamp - ts) / config.recovery_response_time);
            s.velocity += dv;
            // optional: overwrite accel with effective dv/dt
            s.acceleration = dv / dt;
        } else {
            s.velocity += s.acceleration * dt;
        }

        // --- position integration ---
        s.altitude += s.velocity * dt;
        if s.altitude <= config.landing_altitude {
            s.landed = true;
        }
    }

    pub fn current_state(&self) -> PhysicsState {
        self.current_state
    }

    pub fn ignite_engine(&mut self) {
        self.current_state.motor_ignited_ts = Some(self.current_state.timestamp);
        tracing::info!("Engine ignited at t={}", self.current_state.timestamp.get::<second>());
    }

    pub fn deploy_recovery(&mut self) {
        self.current_state.recovery_deployed_ts = Some(self.current_state.timestamp);
        tracing::info!("Recovery deployed at t={}", self.current_state.timestamp.get::<second>());
    }
}
