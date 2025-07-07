#![no_std]
#![no_main]
#![deny(unsafe_code)]

mod io_mapping;
use io_mapping::IOMapping;

use {defmt_rtt as _, panic_probe as _};

use embassy_time::Timer;
use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(get_config());
    let _io_mapping = IOMapping::init(p);

    loop {
        defmt::info!("Hello, world!");
        Timer::after_millis(1000).await;
    }
}

fn get_config() -> embassy_stm32::Config {
    use embassy_stm32::rcc::*;
    use embassy_stm32::rcc::mux::*;

    let mut config = embassy_stm32::Config::default();
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

    config
}
