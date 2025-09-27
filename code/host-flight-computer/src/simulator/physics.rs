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
            current_state: config.initial_state,
        }
    }

    pub fn advance_simulation(&mut self) {
        unimplemented!()
    }

    pub fn current_state(&self) -> PhysicsState {
        self.current_state
    }

    pub fn deploy_recovery(&mut self) {
        unimplemented!()
    }

    pub fn ignite_engine(&mut self) {
        unimplemented!()
    }
}
