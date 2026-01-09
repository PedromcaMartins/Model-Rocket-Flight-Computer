mod finite_state_machine;
pub use finite_state_machine::finite_state_machine_task;
mod sensor;
pub use sensor::sensor_task;
mod storage;
pub use storage::storage_task;
pub mod postcard;
pub use postcard::postcard_server_task;

#[cfg(feature = "impl_software")]
pub mod simulation;