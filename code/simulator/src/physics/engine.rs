use std::sync::Arc;

use proto::uom::si::{
    f32::{Force, Velocity},
    time::second,
};

use crate::{
    config::SimulatorConfig,
    physics::state::PhysicsState,
    types::{ActiveForceEvent, ForceEvent},
};

pub struct PhysicsEngine {
    state: PhysicsState,
    active_events: Arc<ActiveForceEvent>,
}

impl PhysicsEngine {
    pub fn new(active_events: Arc<ActiveForceEvent>) -> Self {
        Self {
            state: PhysicsState::default(),
            active_events,
        }
    }

    pub fn active_events(&self) -> &ActiveForceEvent {
        &self.active_events
    }

    fn total_force(&self) -> Force {
        self.state
            .active_force_events()
            .iter()
            .map(|e| e.compute_force(&self.state))
            .sum()
    }

    pub fn handle_force_event(&mut self, trigger: ForceEvent) {
        match trigger {
            ForceEvent::MotorThrust => {
                if !self.state.is_flying() {
                    self.state.motor_ignited = Some(self.state.time);
                    tracing::info!("ignition triggered at t={:.3}s", self.state.time.get::<second>());
                }
            }
            ForceEvent::Recovery => {
                if self.state.recovery_deployed.is_none() {
                    self.state.recovery_deployed = Some(self.state.time);
                    tracing::info!("deployment triggered at t={:.3}s", self.state.time.get::<second>());
                }
            }
            _ => {}
        }
    }

    pub fn step(&mut self) {
        let dt = proto::uom::si::f32::Time::new::<second>(
            SimulatorConfig::PHYSICS_TIME_STEP_INTERVAL.as_secs_f32(),
        );

        let total_force = self.total_force();
        let mass = SimulatorConfig::rocket_mass();
        let s = &mut self.state;

        s.time += dt;
        s.acceleration = total_force / mass;
        s.velocity += s.acceleration * dt;
        s.altitude += s.velocity * dt;

        let landing_alt = SimulatorConfig::touch_down_altitude();
        if s.altitude <= landing_alt {
            s.altitude = landing_alt;
            s.velocity = Velocity::default();
            s.acceleration = proto::uom::si::f32::Acceleration::default();
            s.touched_down = Some(s.time);
        }

        self.active_events.recompute(s);
    }

    pub fn state(&self) -> PhysicsState {
        self.state.clone()
    }
}
