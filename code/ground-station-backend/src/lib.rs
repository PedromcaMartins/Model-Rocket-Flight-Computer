use std::sync::Arc;
use proto::{PingRequest, PingResponse, actuator_data::ActuatorStatus, record::tick_hz::GlobalTickHz};
use simulator::api::SimulatorClient;
use tokio::{select, sync::Mutex, time::Instant};
use tracing::{info, trace, error};

pub use api::{start_ground_station, ApiConfig};

mod postcard_client;
pub use postcard_client::{PostcardClient, PostcardError};

mod config;
pub use config::*;

mod logging;
pub use logging::{Logging, LoggingConfig};

pub async fn handle_flight_computer(client: PostcardClient) {
    let mut ping = tokio::time::interval(Config::PING_INTERVAL);
    let mut ping_num = 0_u32;
    let mut subscriber_record = client.subscribe_record().await.unwrap();

    let global_tick = client.global_tick().await.unwrap();
    info!("Global tick: {global_tick}");

    loop {
        select! {
            _ = ping.tick() => {
                trace!("Sending ping {ping_num}");
                let _ = client.ping(ping_num.wrapping_add(1).into()).await;
            }
            record = subscriber_record.recv() => {
                match record {
                    Some(record) => trace!("Received record: {record}"),
                    None => error!("Record subscription closed"),
                }
            }
        }
    }
}

pub async fn handle_simulator(postcard: PostcardClient, simulator: SimulatorClient) {
    let mut subscriber_deployment = postcard.subscribe_sim_deployment().await.unwrap();
    let mut altimeter_fut = simulator.clone();
    let mut gps_fut = simulator.clone();
    let mut imu_fut = simulator.clone();
    let mut arm_fut = simulator.clone();

    loop {
        select! {
            data = altimeter_fut.wait_for_altimeter_data() => {
                trace!("Received altimeter data: {data}");
                let _ = postcard.publish_sim_altimeter(&data).await;
            }
            data = gps_fut.wait_for_gps_data() => {
                trace!("Received GPS data: {data}");
                let _ = postcard.publish_sim_gps(&data).await;
            }
            data = imu_fut.wait_for_imu_data() => {
                trace!("Received IMU data: {data}");
                let _ = postcard.publish_sim_imu(&data).await;
            }
            _ = arm_fut.wait_for_arm() => {
                trace!("Received arm command");
                let _ = postcard.publish_sim_arm(&ActuatorStatus::Active).await;
            }
            deployment = subscriber_deployment.recv() => {
                trace!("Received deployment status: {deployment:?}");
                match deployment {
                    Some(ActuatorStatus::Active) => { let _ = simulator.trigger_deployment().await; },
                    Some(ActuatorStatus::Inactive) => info!("Deployment System Reset"),
                    None => error!("Sim deployment subscription closed"),
                }
            }
        }
    }
}
