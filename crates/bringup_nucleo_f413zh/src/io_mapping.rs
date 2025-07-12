
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

use embassy_stm32::{bind_interrupts, gpio::{Input, Level, Output, Pull, Speed}, i2c::{self, I2c}, peripherals::{SDIO, USART2, USART3}, sdmmc::{self, Sdmmc}, time::Hertz, usart::{self, Uart}};

bind_interrupts!(struct Irqs {
    SDIO => sdmmc::InterruptHandler<SDIO>;
    USART2 => usart::InterruptHandler<USART2>;
    USART3 => usart::InterruptHandler<USART3>;
});

pub struct IOMapping {
    pub bno055: Bno055Port,
    pub bmp280: Bmp280Port,
    pub sd_card: SdCardPort,
    pub sd_card_detect: SdCardDetectPort,
    pub sd_card_status_led: SdCardStatusLedPort,
    pub debug_uart: DebugUartPort,
    pub ublox_neo_7m: UbloxNeo7mPort,
}

impl IOMapping {
    pub fn init(p: embassy_stm32::Peripherals) -> Self {
        let (uart_tx, _uart_rx) = defmt::unwrap!(Uart::new(p.USART3, p.PD9, p.PD8, Irqs, p.DMA1_CH3, p.DMA1_CH1, usart::Config::default())).split();

        Self {
            bno055: I2c::new_blocking(p.I2C2, p.PB10, p.PB11, Hertz::khz(400), i2c::Config::default()),
            bmp280: I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Hertz::khz(400), i2c::Config::default()),
            sd_card: Sdmmc::new_1bit(p.SDIO, Irqs, p.DMA2_CH3, p.PC12, p.PD2, p.PC8, sdmmc::Config::default()),
            sd_card_detect: Input::new(p.PG2, Pull::None),
            sd_card_status_led: Output::new(p.PB1, Level::Low, Speed::High),
            debug_uart: uart_tx,
            ublox_neo_7m: defmt::unwrap!(Uart::new(p.USART2, p.PD6, p.PD5, Irqs, p.DMA1_CH6, p.DMA1_CH5, usart::Config::default())),
        }
    }
}
