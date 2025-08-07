
mod types {
    use embassy_stm32::{exti::ExtiInput, gpio::Output, i2c::I2c, mode, peripherals::{DMA2_CH3, SDIO, USB_OTG_FS}, sdmmc::Sdmmc, usart::Uart, usb::Driver};

    pub type Bno055Peripheral = I2c<'static, mode::Blocking>;
    pub type Bmp280Peripheral = I2c<'static, mode::Blocking>;
    pub type SdCardPeripheral = Sdmmc<'static, SDIO, DMA2_CH3>;
    pub type SdCardDetectPeripheral = ExtiInput<'static>;
    pub type SdCardInsertedLedPeripheral = Output<'static>;
    pub type DebugPeripheral = Uart<'static, mode::Async>;
    pub type UbloxNeo7mPeripheral = Uart<'static, mode::Async>;
    pub type PostcardServerUsbDriver = Driver<'static, USB_OTG_FS>;
    pub type InitArmLedPeripheral = Output<'static>;
    pub type RecoveryActivatedLedPeripheral = Output<'static>;
    pub type WarningLedPeripheral = Output<'static>;
    pub type ErrorLedPeripheral = Output<'static>;
    pub type ArmButtonPeripheral = ExtiInput<'static>;
}
use static_cell::ConstStaticCell;
pub use types::*;

use embassy_stm32::{bind_interrupts, exti::ExtiInput, gpio::{Level, Output, Pull, Speed}, i2c::{self, I2c}, peripherals::{SDIO, USART3, USART6, USB_OTG_FS}, sdmmc::{self, Sdmmc}, time::Hertz, usart::{self, Uart}, usb::{self, Driver}};

static EP_OUT_BUFFER: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0u8; 1024]);

bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<USB_OTG_FS>;
    SDIO => sdmmc::InterruptHandler<SDIO>;
    USART6 => usart::InterruptHandler<USART6>;
    USART3 => usart::InterruptHandler<USART3>;
});

pub struct IOMapping {
    pub bno055: Bno055Peripheral,
    pub bmp280: Bmp280Peripheral,
    pub sd_card: SdCardPeripheral,
    pub sd_card_detect: SdCardDetectPeripheral,
    pub sd_card_status_led: SdCardInsertedLedPeripheral,
    pub debug_peripheral: DebugPeripheral,
    pub ublox_neo_7m: UbloxNeo7mPeripheral,
    pub postcard_server_usb_driver: PostcardServerUsbDriver,
    pub init_arm_led: InitArmLedPeripheral,
    pub recovery_activated_led: RecoveryActivatedLedPeripheral,
    pub warning_led: WarningLedPeripheral,
    pub error_led: ErrorLedPeripheral,
    pub arm_button: ArmButtonPeripheral,
}

impl IOMapping {
    pub fn init() -> Self {
        let p = embassy_stm32::init(get_init_config());
        let ep_out_buffer = EP_OUT_BUFFER.take().as_mut_slice();

        Self {
            bno055: I2c::new_blocking(p.I2C2, p.PF1, p.PF0, Hertz::khz(400), i2c::Config::default()),
            bmp280: I2c::new_blocking(p.I2C1, p.PB8, p.PB9, Hertz::khz(400), i2c::Config::default()),
            sd_card: Sdmmc::new_4bit(p.SDIO, Irqs, p.DMA2_CH3, p.PC12, p.PD2, p.PC8, p.PC9, p.PC10, p.PC11, sdmmc::Config::default()),
            sd_card_detect: ExtiInput::new(p.PG2, p.EXTI2, Pull::None),
            sd_card_status_led: Output::new(p.PG3, Level::Low, Speed::High),
            debug_peripheral: defmt::unwrap!(Uart::new(p.USART3, p.PD9, p.PD8, Irqs, p.DMA1_CH3, p.DMA1_CH1, usart::Config::default())),
            ublox_neo_7m: defmt::unwrap!(Uart::new(p.USART6, p.PC7, p.PC6, Irqs, p.DMA2_CH6, p.DMA2_CH1, usart::Config::default())),
            postcard_server_usb_driver: Driver::new_fs(p.USB_OTG_FS, Irqs, p.PA12, p.PA11, ep_out_buffer, usb::Config::default()),
            init_arm_led: Output::new(p.PB0, Level::Low, Speed::High),
            recovery_activated_led: Output::new(p.PB7, Level::Low, Speed::High),
            warning_led: Output::new(p.PA6, Level::Low, Speed::High),
            error_led: Output::new(p.PB14, Level::Low, Speed::High),
            arm_button: ExtiInput::new(p.PC13, p.EXTI13, Pull::None),
        }
    }
}

#[allow(clippy::wildcard_imports)]
fn get_init_config() -> embassy_stm32::Config {
    use embassy_stm32::rcc::*;
    use embassy_stm32::rcc::mux::*;

    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.hse = None;
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.pll_src = PllSource::HSI;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV8,
        mul: PllMul::MUL96,
        divp: Some(PllPDiv::DIV2), // 16mhz / 8 * 96 / 2 = 96Mhz.
        divq: Some(PllQDiv::DIV4), // 16mhz / 8 * 96 / 4 = 48Mhz.
        divr: None,
    });
    config.rcc.plli2s = Some(Pll {
        prediv: PllPreDiv::DIV16,
        mul: PllMul::MUL192,
        divp: None,
        divq: Some(PllQDiv::DIV4), // 16mhz / 16 * 192 / 4 = 48Mhz.
        divr: None,
    });
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.mux.clk48sel = Clk48sel::PLL1_Q;
    config.rcc.mux.sdiosel = Sdiosel::CLK48;
    config.rcc.ls = LsConfig::off();

    config
}
