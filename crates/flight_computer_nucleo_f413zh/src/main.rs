#![no_std]
#![no_main]
#![deny(unsafe_code)]

mod io_mapping;

use {defmt_rtt as _, panic_probe as _};

use embassy_time::Timer;
use io_mapping::IOMapping;
use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _io_mapping = IOMapping::init();

    loop {
        defmt::info!("Hello, world!");
        Timer::after_millis(1000).await;
    }
}
