use std::{fs::File, path::PathBuf};

use chrono::Local;
use tracing::level_filters::LevelFilter;
use tracing_log::LogTracer;
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

pub struct Logging;

pub struct LoggingConfig {
    pub system_log_path: PathBuf,
    pub system_json_log_level: LevelFilter,
    pub system_stdout_log_level: LevelFilter,
    pub flight_computer_log_name: &'static str,
    pub log_dir_path: PathBuf,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let ts = Local::now();
        let ts = ts.format("%Y_%m_%d_%H_%M_%S").to_string();
        let log_dir_path: PathBuf = PathBuf::from("logs").join(&ts);
        Self {
            system_log_path: log_dir_path.join("system.log"),
            system_json_log_level: LevelFilter::DEBUG,
            system_stdout_log_level: LevelFilter::INFO,
            flight_computer_log_name: "flight_computer",
            log_dir_path,
        }
    }
}

impl Logging {
    pub async fn init(config: LoggingConfig) {
        // Capture log::info! messages
        LogTracer::init().expect("Failed to set LogTracer");

        // log files
        tokio::fs::create_dir_all(&config.log_dir_path).await.expect("Failed to create log directory");
        let system_file = File::create_new(&config.system_log_path).expect("Failed to create log file");
        
        // JSON log layer
        let system_json_layer = fmt::layer()
            .json()
            .with_writer(system_file) // log file
            .with_filter(
                EnvFilter::builder()
                    .with_default_directive(config.system_json_log_level.into())
                    .from_env_lossy()
                    .add_directive("flight_computer=OFF".parse().expect("Failed to build EnvFilter"))
            );

        // Stdout log layer
        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_filter(config.system_stdout_log_level);
    
        // Console-subscriber layer
        let console_layer = console_subscriber::spawn();

        // Flight computer specific logging - trace
        let fc_trace_log_file_path = config.log_dir_path.join(format!("{}_TRACE.log", config.flight_computer_log_name));
        let fc_trace_log_file = File::create_new(&fc_trace_log_file_path).expect("Failed to create flight computer log file");
        let fc_trace_layer = fmt::layer()
            .json()
            .with_writer(fc_trace_log_file)
            .with_filter(
                FilterFn::new(|metadata| {
                    metadata.target().starts_with("flight_computer")
                        && metadata.level() == &tracing::Level::TRACE
                })
            );

        // Flight computer specific logging - debug
        let fc_debug_log_file_path = config.log_dir_path.join(format!("{}_DEBUG.log", config.flight_computer_log_name));
        let fc_debug_log_file = File::create_new(&fc_debug_log_file_path).expect("Failed to create flight computer log file");
        let fc_debug_layer = fmt::layer()
            .json()
            .with_writer(fc_debug_log_file)
            .with_filter(
                FilterFn::new(|metadata| {
                    metadata.target().starts_with("flight_computer")
                        && metadata.level() == &tracing::Level::DEBUG
                })
            );

        // Flight computer specific logging - info
        let fc_info_log_file_path = config.log_dir_path.join(format!("{}_INFO.log", config.flight_computer_log_name));
        let fc_info_log_file = File::create_new(&fc_info_log_file_path).expect("Failed to create flight computer log file");
        let fc_info_layer = fmt::layer()
            .json()
            .with_writer(fc_info_log_file)
            .with_filter(
                FilterFn::new(|metadata| {
                    metadata.target().starts_with("flight_computer")
                        && metadata.level() == &tracing::Level::INFO
                })
            );
    
        // Flight computer specific logging - warn
        let fc_warn_log_file_path = config.log_dir_path.join(format!("{}_WARN.log", config.flight_computer_log_name));
        let fc_warn_log_file = File::create_new(&fc_warn_log_file_path).expect("Failed to create flight computer log file");
        let fc_warn_layer = fmt::layer()
            .json()
            .with_writer(fc_warn_log_file)
            .with_filter(
                FilterFn::new(|metadata| {
                    metadata.target().starts_with("flight_computer")
                        && metadata.level() == &tracing::Level::WARN
                })
            );
    
        // Flight computer specific logging - error
        let fc_error_log_file_path = config.log_dir_path.join(format!("{}_ERROR.log", config.flight_computer_log_name));
        let fc_error_log_file = File::create_new(&fc_error_log_file_path).expect("Failed to create flight computer log file");
        let fc_error_layer = fmt::layer()
            .json()
            .with_writer(fc_error_log_file)
            .with_filter(
                FilterFn::new(|metadata| {
                    metadata.target().starts_with("flight_computer")
                        && metadata.level() == &tracing::Level::ERROR
                })
            );
    
        // Combine layers into registry
        let subscriber = registry()
            .with(system_json_layer)
            .with(stdout_layer)
            .with(console_layer)
            .with(fc_trace_layer)
            .with(fc_debug_layer)
            .with(fc_info_layer)
            .with(fc_warn_layer)
            .with(fc_error_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set subscriber");    
    }
}
