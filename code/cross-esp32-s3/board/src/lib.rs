#![no_std]
#![deny(unsafe_code)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

mod types {
    use embedded_hal_bus::spi::ExclusiveDevice;
    use embedded_sdmmc::SdCard;
    use esp_hal::{delay::Delay, gpio::{Input, Output}, i2c::master::I2c, otg_fs::asynch::Driver, spi::master::Spi, uart::Uart, Async, Blocking};
    use esp_hal_smartled::SmartLedsAdapterAsync;
    use switch_hal::{Switch, ActiveHigh};

    pub type Bno055Port = I2c<'static, Blocking>;
    pub type Bmp280Port = I2c<'static, Blocking>;
    pub type SdCardPort = SdCard<ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, Delay>, Delay>;
    pub type SdCardDetectPort = Input<'static>;
    pub type SdCardInsertedLedPort = Output<'static>;
    pub type DebugPort = Uart<'static, Async>;
    pub type UbloxNeo7mPort = Uart<'static, Async>;
    pub type PostcardServerUsbDriver = Driver<'static>;
    pub type ArmButtonPort = Switch<Input<'static>, ActiveHigh>;
    pub type RGBLedPort = SmartLedsAdapterAsync<esp_hal::rmt::ConstChannelAccess<esp_hal::rmt::Tx, 0>, 25>;
}
use defmt::info;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::SdCard;
use esp_hal::{delay::Delay, gpio::{self, Input, Output}, i2c::{self, master::I2c}, otg_fs::{self, asynch::Driver, Usb}, rmt::Rmt, spi::{self, master::Spi}, time::Rate, timer::systimer::SystemTimer, uart::{self, Uart}};
use esp_hal_smartled::{smart_led_buffer, SmartLedsAdapterAsync};
use static_cell::ConstStaticCell;
use switch_hal::IntoSwitch;
pub use types::*;

static EP_OUT_BUFFER: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0u8; 1024]);

pub struct Board {
    pub bno055: Bno055Port,
    pub bmp280: Bmp280Port,
    pub sd_card: SdCardPort,
    pub sd_card_detect: SdCardDetectPort,
    pub sd_card_status_led: SdCardInsertedLedPort,
    pub debug_port: DebugPort,
    pub ublox_neo_7m: UbloxNeo7mPort,
    pub postcard_server_usb_driver: PostcardServerUsbDriver,
    pub arm_button: ArmButtonPort,
    pub rgb_led: RGBLedPort,
}

impl Board {
    pub fn init() -> Self {
        let p = esp_hal::init(get_init_config());
        let ep_out_buffer = EP_OUT_BUFFER.take().as_mut_slice();

        let timer0 = SystemTimer::new(p.SYSTIMER);
        esp_hal_embassy::init(timer0.alarm0);
        info!("Embassy initialized!");

        let rmt = Rmt::new(p.RMT, Rate::from_mhz(80))
            .expect("Failed to initialize RMT")
            .into_async();
        let rmt_buffer = smart_led_buffer!(1);

        Self {
            bno055: I2c::new(
                p.I2C0, 
                i2c::master::Config::default()
                    .with_timeout(i2c::master::BusTimeout::Maximum)
                ).unwrap()
                .with_scl(p.GPIO4)
                .with_sda(p.GPIO5),
            bmp280: I2c::new(p.I2C1, i2c::master::Config::default())
                .unwrap()
                .with_scl(p.GPIO6)
                .with_sda(p.GPIO7),
            sd_card: SdCard::new(
                ExclusiveDevice::new(
                    Spi::new(p.SPI3, spi::master::Config::default()).unwrap()
                        .with_miso(p.GPIO10)
                        .with_sck(p.GPIO11)
                        .with_mosi(p.GPIO12),
                    Output::new(p.GPIO13, gpio::Level::Low, gpio::OutputConfig::default()), 
                    Delay::new(),
                ).unwrap(), 
                Delay::new(),
            ),
            sd_card_detect: Input::new(p.GPIO9, gpio::InputConfig::default()),
            sd_card_status_led: Output::new(p.GPIO14, gpio::Level::Low, gpio::OutputConfig::default()),
            debug_port: Uart::new(p.UART0, uart::Config::default())
                .unwrap()
                .with_rx(p.GPIO44)
                .with_tx(p.GPIO43)
                .into_async(),
            ublox_neo_7m: Uart::new(p.UART2, uart::Config::default())
                .unwrap()
                .with_rx(p.GPIO15)
                .with_tx(p.GPIO16)
                .into_async(),
            postcard_server_usb_driver: Driver::new(
                Usb::new(p.USB0, p.GPIO20, p.GPIO19),
                ep_out_buffer, 
                otg_fs::asynch::Config::default()
            ),
            arm_button: Input::new(p.GPIO21, gpio::InputConfig::default()).into_active_high_switch(),
            rgb_led: SmartLedsAdapterAsync::new(rmt.channel0, p.GPIO48, rmt_buffer),
        }
    }
}

fn get_init_config() -> esp_hal::Config {
    use esp_hal::clock::CpuClock;

    esp_hal::Config::default()
        .with_cpu_clock(
            CpuClock::max()
        )
}
