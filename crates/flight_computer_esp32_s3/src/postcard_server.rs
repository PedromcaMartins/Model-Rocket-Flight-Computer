use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_usb::UsbDevice;
use flight_computer::tasks::postcard::{ping_handler, Context};
use postcard_rpc::{define_dispatch, server::{impls::embassy_usb_v0_4::{dispatch_impl::{WireRxBuf, WireRxImpl, WireSpawnImpl, WireStorage, WireTxImpl}, PacketBuffers}, Server, Dispatch}};
use static_cell::ConstStaticCell;
use telemetry_messages::{PingEndpoint, ENDPOINT_LIST, TOPICS_IN_LIST, TOPICS_OUT_LIST};

use crate::io_mapping::PostcardServerUsbDriver;

type AppStorage = WireStorage<CriticalSectionRawMutex, PostcardServerUsbDriver, 256, 256, 64, 256>;
type BufStorage = PacketBuffers<1024, 1024>;
pub type AppTx = WireTxImpl<CriticalSectionRawMutex, PostcardServerUsbDriver>;
type AppRx = WireRxImpl<PostcardServerUsbDriver>;
type AppServer = Server<AppTx, AppRx, WireRxBuf, MyApp>;

static PBUFS: ConstStaticCell<BufStorage> = ConstStaticCell::new(BufStorage::new());
static STORAGE: AppStorage = AppStorage::new();

define_dispatch! {
    app: MyApp;
    spawn_fn: spawn_fn;
    tx_impl: AppTx;
    spawn_impl: WireSpawnImpl;
    context: Context;

    endpoints: {
        list: ENDPOINT_LIST;

        | EndpointTy                | kind      | handler                       |
        | ----------                | ----      | -------                       |
        | PingEndpoint              | blocking  | ping_handler                  |
    };
    topics_in: {
        list: TOPICS_IN_LIST;

        | TopicTy                   | kind      | handler                       |
        | ----------                | ----      | -------                       |
    };
    topics_out: {
        list: TOPICS_OUT_LIST;
    };
}

pub async fn init_postcard_server(spawner: Spawner, driver: PostcardServerUsbDriver) -> AppServer {
    let pbufs = PBUFS.take();
    let config = embassy_usb_config();

    let context = Context {  };

    let (device, tx_impl, rx_impl) = STORAGE.init(driver, config, pbufs.tx_buf.as_mut_slice());

    // Set timeout to 4ms/frame, instead of the default 2ms/frame
    tx_impl.set_timeout_ms_per_frame(4).await;

    let dispatcher = MyApp::new(context, spawner.into());
    let vkk = dispatcher.min_key_len();

    spawner.must_spawn(usb_task(device));

    Server::new(
        tx_impl,
        rx_impl,
        pbufs.rx_buf.as_mut_slice(),
        dispatcher,
        vkk,
    )
}

/// This handles the server management
#[embassy_executor::task]
pub async fn server_task(mut server: AppServer) {
    loop {
        // If the host disconnects, we'll return an error here.
        // If this happens, just wait until the host reconnects
        let _ = server.run().await;
    }
}

/// This handles the low level USB management
#[embassy_executor::task]
pub async fn usb_task(mut usb: UsbDevice<'static, PostcardServerUsbDriver>) {
    usb.run().await;
}

fn embassy_usb_config() -> embassy_usb::Config<'static> {
    let mut config = embassy_usb::Config::new(0x16c0, 0x27DD);
    config.manufacturer = Some("model_rocket");
    config.product = Some("flight_computer");
    config.serial_number = Some("00000001");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    config
}
