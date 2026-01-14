use proto::{sensor_data::Time, uom::si::{f32::Force, time::second}};
use tokio::time::{Duration, Instant};

use crate::{config::SimulatorConfig, physics::state::PhysicsState, runtime::commands::SimulatorCommand};

impl From<SimulatorCommand> for ActiveCommand {
    fn from(value: SimulatorCommand) -> Self {
        ActiveCommand {
            command: value,
            start_time: Instant::now(),
        }
    }
}

pub struct ActiveCommand {
    command: SimulatorCommand,
    start_time: Instant,
}

impl ActiveCommand {
    fn duration(&self) -> Duration {
        let config = SimulatorConfig::default();
        let time_to_duration = |time_in_seconds: Time| 
            Duration::from_secs_f32(time_in_seconds.get::<second>());

        match self.command {
            SimulatorCommand::Ignition => time_to_duration(config.motor_burn_time),
            SimulatorCommand::Deployment => time_to_duration(config.recovery_response_time),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time);
        elapsed > self.duration()
    }

    pub fn applied_force(&self, state: PhysicsState) -> Force {
        let config = SimulatorConfig::default();

        match self.command {
            SimulatorCommand::Ignition => {
                config.motor_avg_thrust
            },
            SimulatorCommand::Deployment => {
                let desired_v = -config.recovery_terminal_velocity;
                // m * (desired_v - v) / t
                config.rocket_mass * (desired_v - state.velocity) / config.recovery_response_time
            },
        }
    }
}
