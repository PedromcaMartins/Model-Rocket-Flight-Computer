use std::path::PathBuf;
use postcard_rpc::header::VarKeyKind;
use postcard_rpc::host_client::HostClient;
use postcard_rpc::server::impls::test_channels::ChannelWireTx;
use postcard_rpc::{standard_icd::WireError};
use tokio::sync::{mpsc, watch};
use postcard_rpc::test_utils::local_setup;

use crate::simulator::{Simulator, SimulatorConfig};
use crate::sim_devices::{altimeter::SimAltimeter, arm_button::SimButton, deployment_system::SimParachute, gps::SimGps, imu::SimImu, sd_card::{SimSdCard, SimSdCardDetect, SimSdCardStatusLed}};

pub struct SimBoardConfig {
    pub sd_card_log_dir_path: PathBuf,

    pub alt_sd_card_channel_depth: usize,
    pub gps_sd_card_channel_depth: usize,
    pub imu_sd_card_channel_depth: usize,

    pub postcard_fake_server_depth: usize,
    pub postcard_fake_server_err_uri_path: &'static str,
    pub postcard_sender_var_key_kind: VarKeyKind,
}

impl Default for SimBoardConfig {
    fn default() -> Self {
        let sd_card_log_dir_path = PathBuf::from("sd_card_fs");

        Self {
            sd_card_log_dir_path,

            alt_sd_card_channel_depth: 1,
            gps_sd_card_channel_depth: 1,
            imu_sd_card_channel_depth: 1,

            postcard_fake_server_depth: 1024,
            postcard_fake_server_err_uri_path: "/error",
            postcard_sender_var_key_kind: VarKeyKind::Key8,
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
    pub postcard_sender: postcard_rpc::server::Sender<ChannelWireTx>,
    pub postcard_host_client: HostClient<WireError>,
    pub sd_card: SimSdCard,
    pub sd_card_detect: SimSdCardDetect,
    pub sd_card_status_led: SimSdCardStatusLed,
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
            button_tx,
            deployment_rx,
            alt_tx,
            gps_tx,
            imu_tx,

            sd_card_detect_tx,
            sd_card_status_led_rx,

            simulator_config,
        );

        // postcard rpc setup
        let (postcard_local_fake_server, postcard_host_client) = local_setup(
            board_config.postcard_fake_server_depth,
            board_config.postcard_fake_server_err_uri_path
        );
        let postcard_sender = postcard_rpc::server::Sender::new(
            ChannelWireTx::new(postcard_local_fake_server.to_client), 
            board_config.postcard_sender_var_key_kind
        );

        SimBoard {
            simulator,
            arm_button: SimButton::new(button_rx),
            deployment_system: SimParachute::new(deployment_tx),
            altimeter: SimAltimeter::new(alt_rx),
            gps: SimGps::new(gps_rx),
            imu: SimImu::new(imu_rx),
            postcard_sender,
            postcard_host_client,
            sd_card: SimSdCard::new(board_config.sd_card_log_dir_path).await,
            sd_card_detect: SimSdCardDetect::new(sd_card_detect_rx),
            sd_card_status_led: SimSdCardStatusLed::new(sd_card_status_led_tx),
        }
    }
}
