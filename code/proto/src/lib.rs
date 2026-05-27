//! Shared wire vocabulary for the rocket's software stack.
//!
//! All [`postcard-rpc`] Topics, Endpoints, and wire types live here.
//! The crate is `#![no_std]` and feature-gated so embedded targets never
//! compile host-only or simulator-only symbols.
//!
//! See `docs/software/spec.md §9` for the architectural constraints.
//!
//! # Features
//!
//! | Feature | Contents |
//! |---|---|
//! | `default` | `client` (non-transport client utilities) |
//! | `simulator-endpoints` | All `Sim*` topics (altimeter, GPS, IMU, arm, deploy, LEDs) |
//! | `transport-ipc` | [`InterprocessWireTx`], [`InterprocessWireRx`] — tokio + interprocess |
//! | `transport-thread` | In-process channel transport — local server + client pair |
//! | `host` | `simulator-endpoints` + `transport-ipc` + logging — for flight-computer host binary |
//! | `pil` | `simulator-endpoints` + `defmt` — for PIL firmware |
//! | `hw` | `defmt` + `embassy-time` — no sim, no IPC |
//! | `client` | Non-transport client utilities (`PostcardClient`, `PostcardError`) |
//!
//! # Transport features
//!
//! Clients must explicitly enable the transport they use:
//! - `transport-ipc` — interprocess local sockets (tokio + interprocess)
//! - `transport-thread` — in-process mpsc channels (tokio)
//!
//! # Adding a new Topic / Endpoint
//!
//! 1. Add the message type in the appropriate module.
//! 2. Add the `topics!` or `endpoints!` entry below.
//! 3. Gate it by wrapping the entire macro in `#[cfg(feature = "...")]`:
//!    - **Always compiled** (HW-safe) -> no `#[cfg]`.
//!    - **Sim-only** -> `#[cfg(feature = "simulator-endpoints")]`.
//!    - **Transport only** -> `#[cfg(any(feature = "transport-ipc", feature = "transport-thread"))]`.
//!
//! # Verification
//!
//! ```bash
//! # HW: no sim, no IPC
//! cargo clippy --no-default-features --features hw -p proto
//!
//! # Host: flight-computer binary mode
//! cargo clippy --no-default-features --features host -p proto
//!
//! # PIL: sim endpoints, no IPC
//! cargo clippy --no-default-features --features pil -p proto
//!
//! # Client + IPC transport
//! cargo clippy --no-default-features --features "client,transport-ipc" -p proto
//! ```

#![no_std]
#![deny(unsafe_code)]
#![deny(unused_must_use)]

use postcard_schema::schema;
use postcard_rpc::{endpoints, topics, TopicDirection};

pub use serde::{Deserialize, Serialize};
pub use postcard_schema::Schema;
pub use uom;

pub mod sensor_data;
pub mod actuator_data;
pub mod flight_state;
pub mod event;
pub mod error;

mod newtypes;
pub use newtypes::*;

pub mod record;
pub mod wire;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "client")]
pub use client::{PostcardClient, PostcardError};

#[cfg(any(feature = "transport-ipc", feature = "transport-thread"))]
pub mod transport;

use crate::record::Record;
use crate::record::tick_hz::GlobalTickHz;

#[cfg(feature = "simulator-endpoints")]
use crate::{actuator_data::{ActuatorStatus, LedStatus}, flight_state::FlightState, sensor_data::{AltimeterData, GpsData, ImuData}};

/* --------------------------------- Config --------------------------------- */

pub const DEFAULT_SUBSCRIBE_DEPTH: u16 = u16::MAX;

/* ------------------- Postcard RPC Endpoint Configuration ------------------ */

/* --- HW-safe topics (always compiled) --- */

endpoints! {
    list = ENDPOINT_LIST;
    omit_std = true;
    | EndpointTy                | RequestTy         | ResponseTy            | Path                      |
    | ------------------------- | ----------------- | --------------------- | ------------------------- |
    | PingEndpoint              | PingRequest       | PingResponse          | "ping"                    |
    | GlobalTickHzEndpoint      | ()                | GlobalTickHz          | "embassy_time_tick_hz"    |
}

topics! {
    list = TOPICS_GS_OUT_LIST;
    direction = TopicDirection::ToClient;
    | TopicTy                   | MessageTy         | Path                  |
    | ------------------------- | ----------------- | --------------------- |
    | RecordTopic               | Record            | "record"              |
}

/* --- GS -> FC topics (empty — GS uses endpoints, not topics_in) --- */

topics! {
    list = TOPICS_GS_IN_LIST;
    direction = TopicDirection::ToServer;
    omit_std = true;
    | TopicTy                   | MessageTy         | Path                  |
    | ------------------------- | ----------------- | --------------------- |
}

/* --- Simulator-fed topics (gated behind `simulator-endpoints`) --- */

#[cfg(feature = "simulator-endpoints")]
topics! {
    list = TOPICS_SIM_IN_LIST;
    direction = TopicDirection::ToServer;
    | TopicTy                   | MessageTy         | Path                  |
    | ------------------------- | ----------------- | --------------------- |
    | SimAltimeterTopic         | AltimeterData     | "sim_altimeter"       |
    | SimGpsTopic               | GpsData           | "sim_gps"             |
    | SimImuTopic               | ImuData           | "sim_imu"             |
    | SimArmTopic               | ActuatorStatus    | "sim_arm"             |
}

#[cfg(feature = "simulator-endpoints")]
topics! {
    list = TOPICS_SIM_OUT_LIST;
    direction = TopicDirection::ToClient;
    | TopicTy                   | MessageTy         | Path                  |
    | ------------------------- | ----------------- | --------------------- |
    | SimDeploymentTopic        | ActuatorStatus    | "sim_deployment"      |
    | SimFlightStateTopic       | FlightState       | "sim_flight_state"    |
    /* ------------------------------------------ LEDs ----------------------------------------------- */
    | SimPostcardLedTopic       | LedStatus         | "sim_postcard_led"    |
    | SimAltimeterLedTopic      | LedStatus         | "sim_altimeter_led"   |
    | SimGpsLedTopic            | LedStatus         | "sim_gps_led"         |
    | SimImuLedTopic            | LedStatus         | "sim_imu_led"         |
    | SimArmLedTopic            | LedStatus         | "sim_arm_led"         |
    | SimFileSystemLedTopic     | LedStatus         | "sim_file_system_led" |
    | SimDeploymentLedTopic     | LedStatus         | "sim_deployment_led"  |
    | SimGroundStationLedTopic  | LedStatus         | "sim_groundstation_led"|
}
