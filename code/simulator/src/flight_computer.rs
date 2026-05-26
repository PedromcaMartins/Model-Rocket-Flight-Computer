use std::future::pending;
use std::sync::Arc;

use anyhow::Context;
use arc_swap::ArcSwap;
use proto::PostcardClient;
use proto::{
    actuator_data::ActuatorStatus,
    flight_state::FlightState,
    sensor_data::{AltimeterData, GpsData, ImuData},
};
use proto::wire::{
    SimAltimeterLedTopic, SimAltimeterTopic, SimArmLedTopic, SimArmTopic, SimDeploymentLedTopic,
    SimDeploymentTopic, SimFileSystemLedTopic, SimFlightStateTopic, SimGpsLedTopic, SimGpsTopic,
    SimGroundStationLedTopic, SimImuLedTopic, SimImuTopic, SimPostcardLedTopic,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
    time,
};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::{
    config::SimulatorConfig,
    physics::state::PhysicsState,
    types::{ForceEvent, SimActuatorSnapshot},
};

#[derive(Debug, Clone)]
pub enum FcCommand {
    Arm,
}

pub async fn run_fc_client(
    client: Arc<PostcardClient>,
    physics_state_rx: watch::Receiver<PhysicsState>,
    scripted_cmd_rx: mpsc::Receiver<FcCommand>,
    physics_tx: mpsc::Sender<ForceEvent>,
    fc_state_tx: watch::Sender<FlightState>,
    actuator_tx: Arc<ArcSwap<SimActuatorSnapshot>>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    // Spawn subscriber task to receive actuator updates from the FC
    let client_sub = client.clone();
    let cancel_sub = cancel.clone();
    let mut subscriber = tokio::spawn(async move {
        run_subscriber(client_sub, physics_tx, fc_state_tx, actuator_tx, cancel_sub).await
    });

    // Spawn publisher task to send sensor updates and scripted commands to the FC
    let cancel_pub = cancel.clone();
    let mut publisher = tokio::spawn(async move {
        run_publisher(client, physics_state_rx, scripted_cmd_rx, cancel_pub).await
    });

    let check_join = |result, name| {
        match result {
            Ok(inner) => anyhow::bail!("fc-sim {name} exited: FC and simulator desynced: {:?}", inner),
            Err(join_err) => anyhow::bail!("fc-sim {name} panicked: {join_err}"),
        }
    };

    // `biased` so a Ctrl-C cancellation wins over a sub-task that is winding
    // down on the same signal. A sub-task ending while NOT cancelled means the
    // fc-sim pipe broke — that is an unrecoverable desync.
    tokio::select! {
        biased;
        _ = cancel.cancelled() => anyhow::bail!("function cancelled"),
        result = &mut subscriber => check_join(result, "subscriber")?,
        result = &mut publisher => check_join(result, "publisher")?,
    }

    subscriber.abort();
    publisher.abort();
    Ok(())
}

async fn run_publisher(
    client: Arc<PostcardClient>,
    mut physics_state_rx: watch::Receiver<PhysicsState>,
    mut scripted_cmd_rx: mpsc::Receiver<FcCommand>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let mut acquire_ticker = time::interval(SimulatorConfig::DATA_ACQUISITION_INTERVAL);
    let mut arm_handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(pending::<anyhow::Result<()>>());
    let mut scripted_done = false;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => anyhow::bail!("function cancelled"),

            // publish sensor data at a fixed interval
            _ = acquire_ticker.tick() => {
                let state = physics_state_rx.borrow_and_update().clone();
                publish_sensors(&client, state).await?;
            }

            // activate arm state for a fixed duration
            cmd = scripted_cmd_rx.recv(), if !scripted_done => {
                match cmd {
                    Some(FcCommand::Arm) => {
                        let client = client.clone();
                        arm_handle = tokio::spawn(async move {
                            info!("routing Arm command -> SimArmTopic");
                            client.publish::<SimArmTopic>(&ActuatorStatus::Active).await
                                .with_context(|| "publish SimArmTopic::Active failed")?;
                            tokio::time::sleep(SimulatorConfig::ARM_ACTIVE_DELAY).await;
                            client.publish::<SimArmTopic>(&ActuatorStatus::Inactive).await
                                .with_context(|| "publish SimArmTopic::Inactive failed")
                        });
                    },
                    None => scripted_done = true,
                }
            }

            // workaround to handle arm sequence completion asynchronously
            result = &mut arm_handle => {
                match result {
                    Ok(Ok(())) => {
                        info!("arm sequence completed");
                        arm_handle = tokio::spawn(pending::<anyhow::Result<()>>());
                    },
                    Ok(Err(e)) => anyhow::bail!("arm publish failed: {e}"),
                    Err(join_err) => anyhow::bail!("arm task panicked: {join_err}"),
                }
            }
        }
    }
}

async fn publish_sensors(client: &PostcardClient, state: PhysicsState) -> anyhow::Result<()> {
    let altimeter: AltimeterData = state.clone().into();
    let gps: GpsData = state.clone().into();
    let imu: ImuData = state.into();

    client.publish::<SimAltimeterTopic>(&altimeter).await?;
    client.publish::<SimGpsTopic>(&gps).await?;
    client.publish::<SimImuTopic>(&imu).await?;
    Ok(())
}

async fn run_subscriber(
    client: Arc<PostcardClient>,
    physics_tx: mpsc::Sender<ForceEvent>,
    fc_state_tx: watch::Sender<FlightState>,
    actuator_tx: Arc<ArcSwap<SimActuatorSnapshot>>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let mut deploy_sub = client.subscribe::<SimDeploymentTopic>().await?;
    let mut flight_state_sub = client.subscribe::<SimFlightStateTopic>().await?;
    let mut postcard_led_sub = client.subscribe::<SimPostcardLedTopic>().await?;
    let mut altimeter_led_sub = client.subscribe::<SimAltimeterLedTopic>().await?;
    let mut gps_led_sub = client.subscribe::<SimGpsLedTopic>().await?;
    let mut imu_led_sub = client.subscribe::<SimImuLedTopic>().await?;
    let mut arm_led_sub = client.subscribe::<SimArmLedTopic>().await?;
    let mut file_system_led_sub = client.subscribe::<SimFileSystemLedTopic>().await?;
    let mut deployment_led_sub = client.subscribe::<SimDeploymentLedTopic>().await?;
    let mut ground_station_led_sub = client.subscribe::<SimGroundStationLedTopic>().await?;

    macro_rules! recv_actuator {
        ($sub:expr, $field:ident) => {
            async {
                $sub.recv().await
                    .map(|val| {
                        actuator_tx.rcu(|s| SimActuatorSnapshot { $field: val, ..**s });
                    })
                    .context(concat!(stringify!($field), " subscription closed: FC and simulator desynced"))
            }
        };
    }

    loop {
        tokio::select! {
            _ = cancel.cancelled() => anyhow::bail!("function cancelled"),

            // Receive deployment status updates from the FC and sync with physics sim
            status = recv_actuator!(deploy_sub, deployment) => {
                status?;
                info!("deployment activated by FC -> routing Deploy to physics");
                physics_tx.send(ForceEvent::Recovery).await.with_context(|| "physics trigger receiver dropped")?;
            }

            // Receive LED status updates from the FC (non-blocking, writes to shared snapshot)
            status = recv_actuator!(postcard_led_sub, postcard_led)             => { status?; }
            status = recv_actuator!(altimeter_led_sub, altimeter_led)           => { status?; }
            status = recv_actuator!(gps_led_sub, gps_led)                       => { status?; }
            status = recv_actuator!(imu_led_sub, imu_led)                       => { status?; }
            status = recv_actuator!(arm_led_sub, arm_led)                       => { status?; }
            status = recv_actuator!(file_system_led_sub, file_system_led)       => { status?; }
            status = recv_actuator!(deployment_led_sub, deployment_led)         => { status?; }
            status = recv_actuator!(ground_station_led_sub, ground_station_led) => { status?; }

            // Forward flight state updates from the FC to the simulator
            state = flight_state_sub.recv() => {
                let state = state.context("SimFlightStateTopic subscription closed: FC and simulator desynced")?;

                info!("flight state received from FC: {state:?}");
                if fc_state_tx.send(state).is_err() {
                    warn!("no receivers for FC state — discarding flight state update");
                }
            }
        }
    }
}
