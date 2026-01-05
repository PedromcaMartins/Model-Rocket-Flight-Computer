use std::path::PathBuf;
use flight_computer::tasks::postcard::Context;
use ground_station_backend::PostcardClient;
use tokio::sync::{mpsc, watch};

use crate::sim_devices::postcard_server::{LocalServer, postcard_local_setup};
use crate::simulator::{Simulator, SimulatorConfig};
use crate::sim_devices::{altimeter::SimAltimeter, arm_button::SimButton, deployment_system::SimParachute, gps::SimGps, imu::SimImu, sd_card::{SimSdCard, SimSdCardDetect, SimSdCardStatusLed}};
use crate::simulator_ui::SimulatorUi;

pub struct SimBoardConfig {
    pub sd_card_log_dir_path: PathBuf,

    pub alt_sd_card_channel_depth: usize,
    pub gps_sd_card_channel_depth: usize,
    pub imu_sd_card_channel_depth: usize,

    pub postcard_context: Context,
    pub postcard_server_depth: usize,
    pub postcard_server_receive_buffer_size: usize,
}

impl Default for SimBoardConfig {
    fn default() -> Self {
        let sd_card_log_dir_path = PathBuf::from("sd_card_fs");

        Self {
            sd_card_log_dir_path,

            alt_sd_card_channel_depth: 1,
            gps_sd_card_channel_depth: 1,
            imu_sd_card_channel_depth: 1,

            postcard_context: Context {},
            postcard_server_depth: 1024,
            postcard_server_receive_buffer_size: 1024,
        }
    }
}

pub struct SimBoard {
    pub simulator: Simulator,
    pub arm_button: SimButton,
    pub deployment_system: SimParachute,
    pub altimeter: SimAltimeter,
    pub gps: SimGps,
    pub imu: SimImu,
    pub postcard_server: LocalServer,
    pub postcard_client: PostcardClient,
    pub sd_card: SimSdCard,
    pub sd_card_detect: SimSdCardDetect,
    pub sd_card_status_led: SimSdCardStatusLed,
    pub ui: SimulatorUi,
}

impl SimBoard {
    pub async fn init(board_config: SimBoardConfig, simulator_config: SimulatorConfig) -> Self {
        // channels
        let (button_tx, button_rx) = watch::channel(false);
        let (deployment_tx, deployment_rx) = watch::channel(false);
        let (alt_tx, alt_rx) = mpsc::channel(board_config.alt_sd_card_channel_depth);
        let (gps_tx, gps_rx) = mpsc::channel(board_config.gps_sd_card_channel_depth);
        let (imu_tx, imu_rx) = mpsc::channel(board_config.imu_sd_card_channel_depth);

        let (sd_card_detect_tx, sd_card_detect_rx) = watch::channel(false);
        let (sd_card_status_led_tx, sd_card_status_led_rx) = watch::channel(false);

        let simulator = Simulator::new(
            deployment_rx,
            alt_tx,
            gps_tx,
            imu_tx,
            button_tx.clone(),

            simulator_config,
        );

        let ui = SimulatorUi::new(
            button_tx,
            sd_card_detect_tx,
            sd_card_status_led_rx,
        );

        // postcard rpc setup
        let (postcard_server, postcard_client) = postcard_local_setup(
            board_config.postcard_context,
            board_config.postcard_server_depth,
            board_config.postcard_server_receive_buffer_size,
        );

        SimBoard {
            simulator,
            arm_button: SimButton::new(button_rx),
            deployment_system: SimParachute::new(deployment_tx),
            altimeter: SimAltimeter::new(alt_rx),
            gps: SimGps::new(gps_rx),
            imu: SimImu::new(imu_rx),
            postcard_server,
            postcard_client,
            sd_card: SimSdCard::new(board_config.sd_card_log_dir_path).await,
            sd_card_detect: SimSdCardDetect::new(sd_card_detect_rx),
            sd_card_status_led: SimSdCardStatusLed::new(sd_card_status_led_tx),
            ui,
        }
    }
}
