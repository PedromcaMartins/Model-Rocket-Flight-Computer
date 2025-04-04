use std::{f64::consts::PI, path::PathBuf, str::FromStr};

use chrono::NaiveTime;
use defmt_parser::Level;
use groundstation::{parser::{LocationMessage, LogMessage, MessageType, ModulePath}, GroundStation};
use nmea::sentences::FixType;
use telemetry::{AltimeterMessage, GpsMessage, ImuMessage};
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

    let q = 3.0;
    let r_minor = 2.0;

    let time = start_time.elapsed().as_millis() as u64;
    let t = 2.0 * PI * time as f64 / 10_000_f64;
    let z = r_minor * (q * t).sin();

    let altimeter_message = MessageType::AltimeterMessage(AltimeterMessage {
        altitude: z as f32,
        pressure: value,
        temperature: value as f32,
        timestamp: start_time.elapsed().as_micros() as u64,
    });

    LogMessage {
        timestamp: format!("{:.9}", start_time.elapsed().as_secs_f64()),
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

    let p = 2.0;
    let q = 3.0;
    let r = 5.0;
    let r_minor = 2.0;

    let time = start_time.elapsed().as_millis() as u64;
    let t = 2.0 * PI * time as f64 / 10_000_f64;
    let x = (r + r_minor * (q * t).cos()) * (p * t).cos();
    let y = (r + r_minor * (q * t).cos()) * (p * t).sin();

    let gps_message = MessageType::GpsMessage(GpsMessage {
        fix_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        fix_type: FixType::Gps,
        latitude: x,
        longitude: y,
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

    let imu_message = MessageType::ImuMessage(ImuMessage {
        euler_angles: [value, value, value],
        quaternion: [value, value, value, value],
        linear_acceleration: [value, value, value],
        gravity: [value, value, value],
        acceleration: [value, value, value],
        gyro: [value, value, value],
        mag: [value, value, value],
        temperature: value,
        timestamp: time/1_000,
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
