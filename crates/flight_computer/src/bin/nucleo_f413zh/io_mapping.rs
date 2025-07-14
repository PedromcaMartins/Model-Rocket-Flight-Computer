
mod types {
    use embassy_stm32::{gpio::{Input, Output}, i2c::I2c, mode, peripherals::{DMA2_CH3, SDIO}, sdmmc::Sdmmc, usart::{Uart, UartTx}};

    pub type Bno055Port = I2c<'static, mode::Blocking>;
    pub type Bmp280Port = I2c<'static, mode::Blocking>;
    pub type SdCardPort = Sdmmc<'static, SDIO, DMA2_CH3>;
    pub type SdCardDetectPort = Input<'static>;
    pub type SdCardStatusLedPort = Output<'static>;
    pub type DebugUartPort = UartTx<'static, mode::Async>;
    pub type UbloxNeo7mPort = Uart<'static, mode::Async>;
}
pub use types::*;

use embassy_stm32::{bind_interrupts, gpio::{Input, Level, Output, Pull, Speed}, i2c::I2c, peripherals::{SDIO, USART2, USART3}, sdmmc::{self, Sdmmc}, time::Hertz, usart::{self, Uart}};

bind_interrupts!(struct Irqs {
    SDIO => sdmmc::InterruptHandler<SDIO>;
    USART2 => usart::InterruptHandler<USART2>;
    USART3 => usart::InterruptHandler<USART3>;
});

pub struct IOMapping {
    pub bno055_port: Bno055Port,
    pub bmp280_port: Bmp280Port,
    pub sd_card_port: SdCardPort,
    pub sd_card_detect_port: SdCardDetectPort,
    pub sd_card_status_led_port: SdCardStatusLedPort,
    pub debug_uart_port: DebugUartPort,
    pub ublox_neo_7m_port: UbloxNeo7mPort,
}

impl IOMapping {
    pub fn init() -> Self {
        let p = embassy_stm32::init(get_config());
        let (uart_tx, _uart_rx) = defmt::unwrap!(Uart::new(p.USART3, p.PD9, p.PD8, Irqs, p.DMA1_CH3, p.DMA1_CH1, Default::default())).split();

        Self {
            bno055_port: I2c::new_blocking(p.I2C2, p.PB10, p.PB11, Hertz::khz(400), Default::default()),
            bmp280_port: I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Hertz::khz(400), Default::default()),
            sd_card_port: Sdmmc::new_1bit(p.SDIO, Irqs, p.DMA2_CH3, p.PC12, p.PD2, p.PC8, Default::default()),
            sd_card_detect_port: Input::new(p.PG2, Pull::None),
            sd_card_status_led_port: Output::new(p.PB1, Level::Low, Speed::High),
            debug_uart_port: uart_tx,
            ublox_neo_7m_port: defmt::unwrap!(Uart::new(p.USART2, p.PD6, p.PD5, Irqs, p.DMA1_CH6, p.DMA1_CH5, Default::default())),
        }
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
