use embassy_stm32::{bind_interrupts, gpio::{Input, Pull}, i2c::I2c, mode, peripherals::{DMA2_CH3, SDIO, USART2, USART3}, sdmmc::{self, Sdmmc}, time::Hertz, usart::{self, Uart, UartTx}};

bind_interrupts!(struct Irqs {
    SDIO => sdmmc::InterruptHandler<SDIO>;
    USART2 => usart::InterruptHandler<USART2>;
    USART3 => usart::InterruptHandler<USART3>;
});

pub struct IOMapping<'d> {
    pub version: u16,
    pub bno055_i2c: I2c<'d, mode::Blocking>,
    pub bmp280_i2c: I2c<'d, mode::Blocking>,
    pub sd_card: Sdmmc<'d, SDIO, DMA2_CH3>,
    pub sd_card_detect: Input<'d>,
    pub debug_uart: UartTx<'d, mode::Async>,
    pub ublox_neo_7m: Uart<'d, mode::Async>,
}

impl IOMapping<'_> {
    pub fn init(p: embassy_stm32::Peripherals) -> Self {
        let (uart_tx, _uart_rx) = defmt::unwrap!(Uart::new(p.USART3, p.PD9, p.PD8, Irqs, p.DMA1_CH3, p.DMA1_CH1, Default::default())).split();

        Self {
            version: 1,
            bno055_i2c: I2c::new_blocking(p.I2C2, p.PB10, p.PB11, Hertz::khz(400), Default::default()),
            bmp280_i2c: I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Hertz::khz(400), Default::default()),
            sd_card: Sdmmc::new_1bit(p.SDIO, Irqs, p.DMA2_CH3, p.PC12, p.PD2, p.PC8, Default::default()),
            sd_card_detect: Input::new(p.PG2, Pull::None),
            debug_uart: uart_tx,
            ublox_neo_7m: defmt::unwrap!(Uart::new(p.USART2, p.PD6, p.PD5, Irqs, p.DMA1_CH6, p.DMA1_CH5, Default::default())),
        }
    }
}
