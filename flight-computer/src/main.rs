#![no_std]
#![no_main]

mod io_mapping;
mod logger;
mod tasks;
mod drivers;

use io_mapping::IOMapping;
use logger::init_logger_uart;
use tasks::{altimeter, imu, sd_card};
use panic_probe as _;

use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        use embassy_stm32::rcc::mux::*;
        config.rcc.hsi = true;
        config.rcc.hse = None;
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.pll_src = PllSource::HSI;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV8,
            mul: PllMul::MUL100,
            divp: Some(PllPDiv::DIV2), // 16mhz / 8 * 96 / 2 = 96Mhz.
            divq: Some(PllQDiv::DIV4), // 16mhz / 8 * 96 / 4 = 48Mhz.
            divr: None,
        });
        config.rcc.plli2s = Some(Pll { 
            prediv: PllPreDiv::DIV16, 
            mul: PllMul::MUL192, 
            divp: None, 
            divq: Some(PllQDiv::DIV2), // 16mhz / 16 * 192 / 2 = 96Mhz.
            divr: None, 
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;

        config.rcc.mux.sdiosel = Sdiosel::CLK48;
    }
    let p = embassy_stm32::init(config);
    let io_mapping = IOMapping::init(p);

    init_logger_uart(io_mapping.telemetry_uart);

    defmt::unwrap!(spawner.spawn(imu(io_mapping.bno055_i2c)));
    defmt::unwrap!(spawner.spawn(sd_card(io_mapping.sd_card)));
    defmt::unwrap!(spawner.spawn(altimeter(io_mapping.bmp280_i2c)));
}
