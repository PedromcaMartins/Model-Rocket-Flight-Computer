#![allow(clippy::unused_async)]

mod finite_state_machine;
pub use finite_state_machine::finite_state_machine_task;

mod bno055;
pub use bno055::bno055_task;

mod bmp280;
pub use bmp280::bmp280_task;

mod gps;
pub use gps::gps_task;

mod arm_button;
pub use arm_button::arm_button_task;

pub mod postcard;
