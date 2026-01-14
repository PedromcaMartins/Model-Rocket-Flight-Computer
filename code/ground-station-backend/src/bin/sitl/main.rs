use flight_computer::tasks::simulation::start_sil_flight_computer;
use ground_station_backend::{ApiConfig, GroundStationConfig, Logging, handle_flight_computer, handle_simulator, start_ground_station};
use tokio::select;
use tracing::error;

use crate::postcard_server::postcard_local_setup;

mod postcard_server;

#[tokio::main]
pub async fn main() {
    let config = GroundStationConfig::default();
    Logging::init(config.logging).await;

    let (postcard_server, postcard_client) = postcard_local_setup(config.postcard);

    let flight_computer = tokio::spawn(start_sil_flight_computer(server));
    let simulator_client = simulator::start();

    let gs_handle_fc = tokio::spawn(handle_flight_computer(postcard_client.clone()));
    let gs_handle_sim = tokio::spawn(handle_simulator(postcard_client.clone(), simulator_client));

    select! {
        _ = flight_computer => {
            error!("Flight computer task ended");
        }
        _ = gs_handle_fc => {
            error!("Ground station flight computer handler ended");
        }
        _ = gs_handle_sim => {
            error!("Ground station simulator handler ended");
        }
    }
}
