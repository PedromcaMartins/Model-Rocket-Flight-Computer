use std::{fs, io};
use std::path::PathBuf;

use chrono::Local;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

pub struct LoggingGuard {
    _guards: Vec<WorkerGuard>,
}

pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(%info, "process panicked");
        default_hook(info);
    }));
}

pub fn init_tracing() -> Result<LoggingGuard, Box<dyn std::error::Error>> {
    let log_dir = log_dir();
    fs::create_dir_all(&log_dir)?;

    let mut guards = Vec::new();

    let stdout_layer = fmt::layer()
        .with_writer(io::stdout)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        );

    let (info_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "info.json"),
    );
    guards.push(g);
    let info_layer = fmt::layer()
        .json()
        .with_writer(info_w)
        .with_filter(FilterFn::new(|metadata| metadata.level() == &tracing::Level::INFO));

    let (debug_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "debug.json"),
    );
    guards.push(g);
    let debug_layer = fmt::layer()
        .json()
        .with_writer(debug_w)
        .with_filter(FilterFn::new(|metadata| metadata.level() == &tracing::Level::DEBUG));

    let (warn_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "warn.json"),
    );
    guards.push(g);
    let warn_layer = fmt::layer()
        .json()
        .with_writer(warn_w)
        .with_filter(FilterFn::new(|metadata| metadata.level() == &tracing::Level::WARN));

    let (error_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "error.json"),
    );
    guards.push(g);
    let error_layer = fmt::layer()
        .json()
        .with_writer(error_w)
        .with_filter(FilterFn::new(|metadata| metadata.level() == &tracing::Level::ERROR));

    let (trace_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "trace.json"),
    );
    guards.push(g);
    let trace_layer = fmt::layer()
        .json()
        .with_writer(trace_w)
        .with_filter(FilterFn::new(|metadata| metadata.level() == &tracing::Level::TRACE));

    let (all_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "log.json"),
    );
    guards.push(g);
    let all_layer = fmt::layer()
        .json()
        .with_writer(all_w)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::TRACE.into())
                .from_env_lossy(),
        );

    let subscriber = registry()
        .with(stdout_layer)
        .with(info_layer)
        .with(debug_layer)
        .with(warn_layer)
        .with(error_layer)
        .with(trace_layer)
        .with(all_layer);

    tracing::subscriber::set_global_default(subscriber)?;
    tracing_log::LogTracer::init()?;

    tracing::info!(log_dir = %log_dir.display(), "logging initialized");

    Ok(LoggingGuard { _guards: guards })
}

fn log_dir() -> PathBuf {
    let ts = Local::now();
    let ts = ts.format("%Y_%m_%d_%H_%M_%S").to_string();
    PathBuf::from("logs").join(&ts)
}
