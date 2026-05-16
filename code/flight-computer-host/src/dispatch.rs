use flight_computer::tasks::postcard::{
    embassy_time_tick_hz_handler, ping_handler, Context,
};
use flight_computer::tasks::postcard::simulator::{
    sim_altimeter_update, sim_arming_activate, sim_gps_update, sim_imu_update,
};
use postcard_rpc::define_dispatch;
use proto::{
    ENDPOINT_LIST, GlobalTickHzEndpoint, PingEndpoint, SimAltimeterTopic,
    SimArmTopic, SimGpsTopic, SimImuTopic, TOPICS_GS_IN_LIST,
    TOPICS_GS_OUT_LIST, TOPICS_SIM_IN_LIST,
    TOPICS_SIM_OUT_LIST,
};

pub(crate) mod sim {
    use super::*;
    #[allow(unused_imports)]
    use postcard_rpc::server::impls::test_channels::{tokio_spawn, ChannelWireSpawn};

    define_dispatch! {
        app: SimDispatch;
        spawn_fn: tokio_spawn;
        tx_impl: proto::ipc_adapter::InterprocessWireTx;
        spawn_impl: ChannelWireSpawn;
        context: Context;

        endpoints: {
            list: ENDPOINT_LIST;

               | EndpointTy        | kind      | handler                  |
               | -                 | -         | -                        |
        };
        topics_in: {
            list: TOPICS_SIM_IN_LIST;

               | TopicTy           | kind      | handler                  |
               | -                 | -         | -                        |
               | SimAltimeterTopic | blocking  | sim_altimeter_update     |
               | SimGpsTopic       | blocking  | sim_gps_update           |
               | SimImuTopic       | blocking  | sim_imu_update           |
               | SimArmTopic       | blocking  | sim_arming_activate      |
        };
        topics_out: {
            list: TOPICS_SIM_OUT_LIST;
    
            // | TopicTy                   | MessageTy         | Path                  |
            // | ------------------------- | ----------------- | --------------------- |
            // | SimDeploymentTopic        | ActuatorStatus    | "sim_deployment"      |
            // /* ------------------------------------------ LEDs ----------------------------------------------- */
            // | SimPostcardLedTopic       | LedStatus         | "sim_postcard_led"    |
            // | SimAltimeterLedTopic      | LedStatus         | "sim_altimeter_led"   |
            // | SimGpsLedTopic            | LedStatus         | "sim_gps_led"         |
            // | SimImuLedTopic            | LedStatus         | "sim_imu_led"         |
            // | SimArmLedTopic            | LedStatus         | "sim_arm_led"         |
            // | SimFileSystemLedTopic     | LedStatus         | "sim_file_system_led" |
            // | SimDeploymentLedTopic     | LedStatus         | "sim_deployment_led"  |
            // | SimGroundStationLedTopic  | LedStatus         | "sim_groundstation_led"|
        };
    }
}

pub(crate) mod gs {
    use super::*;
    #[allow(unused_imports)]
    use postcard_rpc::server::impls::test_channels::{tokio_spawn, ChannelWireSpawn};

    define_dispatch! {
        app: GsDispatch;
        spawn_fn: tokio_spawn;
        tx_impl: proto::ipc_adapter::InterprocessWireTx;
        spawn_impl: ChannelWireSpawn;
        context: Context;

        endpoints: {
            list: ENDPOINT_LIST;

               | EndpointTy           | kind       | handler                      |
               | -                    | -          | -                            |
               | PingEndpoint         | blocking   | ping_handler                 |
               | GlobalTickHzEndpoint | blocking   | embassy_time_tick_hz_handler |
        };
        topics_in: {
            list: TOPICS_GS_IN_LIST;

               | TopicTy              | kind       | handler                      |
               | -                    | -          | -                            |
        };
        topics_out: {
            list: TOPICS_GS_OUT_LIST;

            // | TopicTy                   | MessageTy         | Path                  |
            // | ------------------------- | ----------------- | --------------------- |
            // | RecordTopic               | Record            | "record"              |
        };
    }
}
