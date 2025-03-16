
pub mod types {
    use embassy_stm32::{mode, peripherals::{DMA2_CH3, I2C1, SDIO}};

    pub type Bno055I2cMode = mode::Blocking;

    pub type Bmp280I2cMode = mode::Blocking;

    pub type SdCard = SDIO;
    pub type SdCardDma = DMA2_CH3;
}
pub use types::*;

use embassy_stm32::{bind_interrupts, gpio::{Input, Pull}, i2c::I2c, peripherals::SDIO, sdmmc::{self, Sdmmc}, time::Hertz};

bind_interrupts!(struct Irqs {
    SDIO => sdmmc::InterruptHandler<SdCard>;
});

pub struct IOMapping<'d> {
    pub version: u16,
    pub bno055_i2c: I2c<'d, Bno055I2cMode>,
    pub bmp280_i2c: I2c<'d, Bmp280I2cMode>,
    pub sd_card: Sdmmc<'d, SdCard, SdCardDma>,
    pub sd_card_detect: Input<'d>,
}

impl IOMapping<'_> {
    pub fn init(p: embassy_stm32::Peripherals) -> Self {
        (
            Self {
                version: 1,
                bno055_i2c: I2c::new_blocking(p.I2C2, p.PB10, p.PB11, Hertz::khz(400), Default::default()),
                bmp280_i2c: I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Hertz::khz(400), Default::default()),
                sd_card: Sdmmc::new_1bit(p.SDIO, Irqs, p.DMA2_CH3, p.PC12, p.PD2, p.PC8, Default::default()),
                sd_card_detect: Input::new(p.PG2, Pull::None),
            }
        )
    }
}
