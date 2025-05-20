use std::{path::PathBuf, str::FromStr};

use chrono::NaiveTime;
use defmt_parser::Level;
use groundstation::{parser::{LocationMessage, LogMessage, MessageType, ModulePath}, GroundStation};
use nalgebra::{UnitQuaternion, Vector3};
use nmea::sentences::FixType;
use telemetry_messages::{AltimeterMessage, GpsMessage, ImuMessage};
use time::OffsetDateTime;
use tokio::{sync::mpsc, time::Instant};

#[tokio::main]
async fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Spawn the GUI in a separate thread
    eframe::run_native(
        "Ground Station",
        Default::default(),
        Box::new(|cc| {            
            let groundstation = GroundStation::new(cc);
            tokio::spawn(simulated_telem(groundstation.clone_tx()));
            Ok(Box::new(groundstation))
        }),
    )
}

async fn simulated_telem(tx: mpsc::Sender<LogMessage>) {
    let start_time = Instant::now();
    loop {
        tx.send(simulate_string_message(&start_time)).await.ok();
        tx.send(simulate_altimeter_message(&start_time)).await.ok();
        tx.send(simulate_gps_message(&start_time)).await.ok();
        tx.send(simulate_imu_message(&start_time)).await.ok();

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

fn simulate_string_message(start_time: &Instant) -> LogMessage {
    let host_timestamp = OffsetDateTime::now_utc()
    .unix_timestamp_nanos()
    .min(i64::MAX as i128) as i64;

    LogMessage {
        timestamp: format!("{:.9}", start_time.elapsed().as_secs_f64()),
        host_timestamp,
        level: Some(Level::Info),
        message: MessageType::String("Hello World!".to_string()),
        location: Some(LocationMessage {
            file_complete_path: PathBuf::from_str("src/bin/mock.rs").unwrap(),
            file: "bin/mock.rs".to_string(),
            line: 34,
            module_path: Some(ModulePath {
                crate_name: "groundstation".to_string(),
                modules: vec!["mock".to_string()],
                function: "simulated_telem".to_string(),
            }),
        }),
    }
}

fn simulate_altimeter_message(start_time: &Instant) -> LogMessage {
    let host_timestamp = OffsetDateTime::now_utc()
    .unix_timestamp_nanos()
    .min(i64::MAX as i128) as i64;

    let time = start_time.elapsed().as_nanos() as u64;
    let value = (time as f64 * 2.0).sin(); // Simulated telemetry data (sine wave)

    let time = start_time.elapsed().as_millis() as f32 / 1_000_f32;
    let (position, _) = generate_pose(time);
    let altitude = position.z;

    let altimeter_message = MessageType::AltimeterMessage(AltimeterMessage {
        altitude,
        pressure: value,
        temperature: value as f32,
        timestamp: start_time.elapsed().as_micros() as u64,
    });

    LogMessage {
        timestamp: format!("{:.9}", start_time.elapsed().as_micros()),
        host_timestamp,
        level: Some(Level::Info),
        message: altimeter_message,
        location: Some(LocationMessage {
            file_complete_path: PathBuf::from_str("src/bin/mock.rs").unwrap(),
            file: "bin/mock.rs".to_string(),
            line: 58,
            module_path: Some(ModulePath {
                crate_name: "groundstation".to_string(),
                modules: vec!["mock".to_string()],
                function: "simulate_altimeter_message".to_string(),
            }),
        }),
    }
}

fn simulate_gps_message(start_time: &Instant) -> LogMessage {
    let host_timestamp = OffsetDateTime::now_utc()
    .unix_timestamp_nanos()
    .min(i64::MAX as i128) as i64;

    let time = start_time.elapsed().as_nanos() as u64;
    let value = (time as f64 * 2.0).sin(); // Simulated telemetry data (sine wave)

    let time = start_time.elapsed().as_millis() as f32 / 1_000_f32;
    let (position, _) = generate_pose(time);
    let (latitude, longitude) = (position.x as f64, position.y as f64);

    let gps_message = MessageType::GpsMessage(GpsMessage {
        fix_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        fix_type: FixType::Gps,
        latitude,
        longitude,
        altitude: value as f32,
        num_of_fix_satellites: 10,
        timestamp: start_time.elapsed().as_micros() as u64,
    });

    LogMessage {
        timestamp: format!("{:.9}", start_time.elapsed().as_secs_f64()),
        host_timestamp,
        level: Some(Level::Info),
        message: gps_message,
        location: Some(LocationMessage {
            file_complete_path: PathBuf::from_str("src/bin/mock.rs").unwrap(),
            file: "bin/mock.rs".to_string(),
            line: 91,
            module_path: Some(ModulePath {
                crate_name: "groundstation".to_string(),
                modules: vec!["mock".to_string()],
                function: "simulate_gps_message".to_string(),
            }),
        }),
    }
}

fn simulate_imu_message(start_time: &Instant) -> LogMessage {
    let host_timestamp = OffsetDateTime::now_utc()
    .unix_timestamp_nanos()
    .min(i64::MAX as i128) as i64;

    let time = start_time.elapsed().as_nanos() as u64;
    let value = (time as f64 * 2.0).sin() as f32; // Simulated telemetry data (sine wave)

    let time = start_time.elapsed().as_millis() as f32 / 1_000_f32;
    let (_, quaternion) = generate_pose(time);

    let euler_angles = quaternion.euler_angles();
    let euler_angles = [
        euler_angles.0.to_degrees(),
        euler_angles.1.to_degrees(),
        euler_angles.2.to_degrees(),
    ];

    let imu_message = MessageType::ImuMessage(ImuMessage {
        euler_angles,
        quaternion: [quaternion.i, quaternion.j, quaternion.k, quaternion.w],
        linear_acceleration: [value, value, value],
        gravity: [value, value, value],
        acceleration: [value, value, value],
        gyro: [value, value, value],
        mag: [value, value, value],
        temperature: value,
        timestamp: start_time.elapsed().as_micros() as u64,
    });

    LogMessage {
        timestamp: format!("{:.9}", start_time.elapsed().as_secs_f64()),
        host_timestamp,
        level: Some(Level::Info),
        message: imu_message,
        location: Some(LocationMessage {
            file_complete_path: PathBuf::from_str("src/bin/mock.rs").unwrap(),
            file: "bin/mock.rs".to_string(),
            line: 124,
            module_path: Some(ModulePath {
                crate_name: "groundstation".to_string(),
                modules: vec!["mock".to_string()],
                function: "simulate_imu_message".to_string(),
            }),
        }),
    }
}

/// Simulates realistic motion of a body using periodic functions.
///
/// # Arguments
/// * `time` - A floating-point value representing time (in seconds).
///
/// # Returns
/// * `(Vector3<f32>, UnitQuaternion<f32>)` - A tuple containing position and orientation as a quaternion.
fn generate_pose(time: f32) -> (Vector3<f32>, UnitQuaternion<f32>) {
    let radius = 5.0; // Radius of the circular path
    let angular_velocity = 1.0; // Angular velocity (radians per second)

    // Parametric equations for a point on a circle (xy-plane)
    let x = radius * (angular_velocity * time).cos(); // longitude
    let y = radius * (angular_velocity * time).sin(); // latitude
    let z = 0.0; // Altitude (fixed at 0 for simplicity)

    let position = Vector3::new(x, y, z);

    // The body rotates around the z-axis with a given angular velocity
    let angle = angular_velocity * time;
    let orientation = UnitQuaternion::from_euler_angles(0.0, angle, 0.0); // Rotation around the Z-axis

    (position, orientation)
}


