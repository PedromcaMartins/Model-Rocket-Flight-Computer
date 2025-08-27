#![allow(clippy::unused_async)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::unwrap_used)]

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

mod sd_card;
pub use sd_card::sd_card_task;

mod system_status;
pub use system_status::system_status_task;

pub mod postcard;
