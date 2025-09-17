use std::{fs::File, path::PathBuf};

use chrono::Local;
use tracing::level_filters::LevelFilter;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

pub struct Logging;

pub struct LoggingConfig {
    pub tracing_log_path: PathBuf,
    pub tracing_log_dir_path: PathBuf,
    pub json_layer_log_level: LevelFilter,
    pub stdout_layer_log_level: LevelFilter,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let ts = Local::now();
        let ts = ts.format("%Y_%m_%d_%H_%M_%S").to_string();
        let tracing_log_dir_path: PathBuf = PathBuf::from("logs");
        Self {
            tracing_log_path: tracing_log_dir_path.join(format!("tracing_{ts}.log")),
            tracing_log_dir_path,
            json_layer_log_level: LevelFilter::DEBUG,
            stdout_layer_log_level: LevelFilter::INFO,
        }
    }
}

impl Logging {
    pub async fn init(config: LoggingConfig) {
        // Capture log::info! messages
        LogTracer::init().expect("Failed to set LogTracer");

        // log file
        tokio::fs::create_dir_all(&config.tracing_log_dir_path).await.expect("Failed to create log directory");
        let file = File::create_new(&config.tracing_log_path).expect("Failed to create log file");
    
        // JSON log layer
        let json_layer = fmt::layer()
            .json()
            .with_writer(file) // log file
            .with_filter(
                EnvFilter::builder()
                    .with_default_directive(config.json_layer_log_level.into())
                    .from_env_lossy(),
            );
    
        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .with_filter(config.stdout_layer_log_level);
    
        // Console-subscriber layer
        let console_layer = console_subscriber::spawn();
    
        // Combine layers into registry
        let subscriber = registry()
            .with(json_layer)
            .with(stdout_layer)
            .with(console_layer);
    
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set subscriber");    
    }
}