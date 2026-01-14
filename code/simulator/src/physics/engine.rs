use proto::{sensor_data::Velocity, uom::si::f32::Force};

use super::{state::PhysicsState};
use crate::{config::SimulatorConfig, physics::events::ActiveCommand, runtime::commands::SimulatorCommand};

#[derive(Default)]
pub struct PhysicsEngine {
    state: PhysicsState,
    config: SimulatorConfig,
    /// List of active commands with their start time
    active_commands: Vec<ActiveCommand>,
}

impl PhysicsEngine {
    fn remove_expired_commands(&mut self) {
        self.active_commands.retain(|command| {
            !command.is_expired()
        });
    }

    fn total_force(&mut self) -> Force {
        // --- skip simulation if motor hasnt been ignited or rocket landed ---
        if self.state.motor_ignited.is_none() || self.state.landed {
            return Force::default();
        }

        let mut total = -self.config.gravity;

        self.remove_expired_commands();
        for command in &self.active_commands {
            total += command.applied_force(self.state());
        }

        total
    }

    pub fn handle_command(&mut self, command: SimulatorCommand) {
        self.active_commands.push(ActiveCommand::from(command));
    }

    pub fn step(&mut self) {
        // calculate total of forces and integrate
        let total_force  = self.total_force();
        let config = &self.config;
        let s = &mut self.state;
        let dt = config.time_step;

        // advance simulation time
        s.time += dt;

        s.acceleration = total_force / config.rocket_mass;
        s.velocity += s.acceleration * dt;
        s.altitude += s.velocity * dt;

        // check for landing
        let landing_altitude = config.landing_altitude;
        if s.altitude <= landing_altitude {
            s.altitude = landing_altitude;
            s.velocity = Velocity::default();
            s.landed = true;
        }
    }

    pub fn state(&self) -> PhysicsState {
        self.state.clone()
    }
}
