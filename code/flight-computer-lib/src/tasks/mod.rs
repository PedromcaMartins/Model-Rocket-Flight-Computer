mod finite_state_machine;
pub use finite_state_machine::finite_state_machine_task;

mod imu;
pub use imu::imu_task;

mod altimeter;
pub use altimeter::altimeter_task;

mod gps;
pub use gps::gps_task;

mod sd_card;
pub use sd_card::sd_card_task;

pub mod postcard;
