use proto::sensor_data::{AltimeterData, GpsData, ImuData};
use tokio::sync::{mpsc, broadcast};
use crate::{physics::state::PhysicsState, runtime::commands::{FlightComputerCommand, SimulatorCommand}};

pub struct ApiHandle {
    simulator_tx: mpsc::Sender<SimulatorCommand>,
    flight_computer_rx: broadcast::Receiver<FlightComputerCommand>,
    state_rx: broadcast::Receiver<PhysicsState>,
}

impl ApiHandle {
    pub fn new(
        simulator_tx: mpsc::Sender<SimulatorCommand>,
        flight_computer_rx: broadcast::Receiver<FlightComputerCommand>,
        state_rx: broadcast::Receiver<PhysicsState>,
    ) -> Self {
        Self {
            simulator_tx,
            flight_computer_rx,
            state_rx,
        }
    }

    async fn wait_for_physics_state(&mut self) -> PhysicsState {
        self.state_rx.recv().await.expect("Failed to receive PhysicsState from broadcast channel")
    }

    pub async fn wait_for_altimeter_data(&mut self) -> AltimeterData {
        AltimeterData::from(self.wait_for_physics_state().await)
    }

    pub async fn wait_for_gps_data(&mut self) -> GpsData {
        GpsData::from(self.wait_for_physics_state().await)
    }

    pub async fn wait_for_imu_data(&mut self) -> ImuData {
        ImuData::from(self.wait_for_physics_state().await)
    }

    async fn wait_for_flight_computer_command(&mut self) -> FlightComputerCommand {
        self.flight_computer_rx.recv().await.expect("Failed to receive FlightComputerCommand from broadcast channel")
    }

    pub async fn wait_for_arm(&mut self) {
        loop {
            if self.wait_for_flight_computer_command().await == FlightComputerCommand::Arm {
                return;
            }
        } 
    }

    async fn send_command(&self, physics_command: SimulatorCommand) {
        if let Err(err) = self.simulator_tx.send(physics_command).await {
            log::error!("Send Physics command Error: {err:?}");
        }
    }

    pub async fn trigger_ignition(&self) {
        self.send_command(SimulatorCommand::Ignition).await;
    }

    pub async fn trigger_deployment(&self) {
        self.send_command(SimulatorCommand::Deployment).await;
    }
}
