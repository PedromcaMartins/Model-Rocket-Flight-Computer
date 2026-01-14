#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulatorCommand {
    Ignition,
    Deployment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlightComputerCommand {
    Arm,
}
