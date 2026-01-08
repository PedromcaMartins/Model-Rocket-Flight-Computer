#![no_std]
#![deny(unsafe_code)]
#![deny(unused_must_use)]

use postcard_schema::{Schema, schema};
use postcard_rpc::{endpoints, topics, TopicDirection};

pub use serde::{Deserialize, Serialize};
pub use uom;

pub mod sensor_data;
pub mod actuator_data;
pub mod flight_state;
pub mod event;
pub mod error;

mod newtypes;
pub use newtypes::*;

mod record;
pub use record::*;

use crate::{actuator_data::{ActuatorStatus, LedStatus}, error::Error, event::Event, flight_state::FlightState, sensor_data::{AltimeterData, GpsData, ImuData}};


/* ------------------- Postcard RPC Endpoint Configuration ------------------ */

endpoints! {
    list = ENDPOINT_LIST;
    omit_std = true;
    | EndpointTy                | RequestTy         | ResponseTy            | Path          |
    | ----------                | ---------         | ----------            | ----          |
    | PingEndpoint              | PingRequest       | PingResponse          | "ping"        |
}

topics! {
    list = TOPICS_IN_LIST;
    direction = TopicDirection::ToServer;
    | TopicTy                   | MessageTy         | Path                  | Cfg           |
    | -------                   | ---------         | ----                  | ---           |
    | SimAltimeterTopic         | AltimeterData     | "sim_altimeter"       |               |
    | SimGpsTopic               | GpsData           | "sim_gps"             |               |
    | SimImuTopic               | ImuData           | "sim_imu"             |               |
    | SimArmTopic               | ActuatorStatus    | "sim_arm"             |               |
}

topics! {
    list = TOPICS_OUT_LIST;
    direction = TopicDirection::ToClient;
    | TopicTy                   | MessageTy         | Path                  | Cfg           |
    | -------                   | ---------         | ----                  | ---           |
    | RecordTopic               | Record            | "record"              |               |
    | FlightStateTopic          | FlightState       | "flight_state"        |               |
    | EventTopic                | Event             | "event"               |               |
    | ErrorTopic                | Error             | "error"               |               |
    | SimDeploymentTopic        | ActuatorStatus    | "sim_deployment"      |               |
    | SimFileSystemLedTopic     | LedStatus         | "sim_file_sytem_led"  |               |
}
