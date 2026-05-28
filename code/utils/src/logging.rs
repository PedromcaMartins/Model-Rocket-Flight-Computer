use std::collections::VecDeque;
use std::io;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use chrono::Local;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry;
use tracing_subscriber::EnvFilter;

use crate::constants::{DEFAULT_BUFFER_CAPACITY, TIMESTAMP_FORMAT};

// ---------------------------------------------------------------------------
// TUI ring buffer
// ---------------------------------------------------------------------------

pub static LOG_BUFFER: LazyLock<Mutex<VecDeque<String>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

#[derive(Clone)]
pub struct BufferMakeWriter;

impl io::Write for BufferMakeWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Ok(text) = std::str::from_utf8(bytes) {
            let mut guard = LOG_BUFFER.lock().unwrap_or_else(|p| p.into_inner());
            for line in text.lines() {
                if !line.is_empty() {
                    guard.push_back(line.to_string());
                }
            }
            while guard.len() > DEFAULT_BUFFER_CAPACITY {
                guard.pop_front();
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for BufferMakeWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        BufferMakeWriter
    }
}

// ---------------------------------------------------------------------------
// UI configuration
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum UiConfig {
    Stdout,
    TuiBuffer,
}

// ---------------------------------------------------------------------------
// Logging configuration
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct LogConfig {
    pub log_root: PathBuf,
    pub stdout_level: tracing::level_filters::LevelFilter,
    pub ui: UiConfig,
}

// ---------------------------------------------------------------------------
// Guard
// ---------------------------------------------------------------------------

pub struct LoggingGuard {
    pub _guards: Vec<WorkerGuard>,
    pub log_dir: PathBuf,
}

// ---------------------------------------------------------------------------
// Custom timer for TUI output
// ---------------------------------------------------------------------------

struct SubsecondTimer;

impl FormatTime for SubsecondTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%H:%M:%S%.3f"))
    }
}

// ---------------------------------------------------------------------------
// Panic hook
// ---------------------------------------------------------------------------

pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(%info, "process panicked");
        default_hook(info);
    }));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn timestamp_dir() -> String {
    Local::now().format(TIMESTAMP_FORMAT).to_string()
}

fn make_file_layer<S>(
    log_dir: &std::path::Path,
    name: &'static str,
    filter_fn: fn(&tracing::Metadata<'_>) -> bool,
    guards: &mut Vec<WorkerGuard>,
) -> impl tracing_subscriber::Layer<S> + Send + Sync
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    let (non_blocking, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::never(log_dir, name));
    guards.push(guard);
    fmt::layer()
        .json()
        .with_writer(BoxMakeWriter::new(non_blocking))
        .with_filter(FilterFn::new(filter_fn))
}

// ---------------------------------------------------------------------------
// Initialisation
// ---------------------------------------------------------------------------

pub fn init_tracing(config: LogConfig) -> anyhow::Result<LoggingGuard> {
    let log_dir = config.log_root.join(timestamp_dir());
    std::fs::create_dir_all(&log_dir)?;

    let mut guards = Vec::new();

    // UI layer
    let ui_layer: Box<dyn tracing_subscriber::Layer<registry::Registry> + Send + Sync> =
        match config.ui {
            UiConfig::Stdout => fmt::layer()
                .with_writer(io::stdout)
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .boxed(),
            UiConfig::TuiBuffer => fmt::layer()
                .with_writer(BufferMakeWriter)
                .with_timer(SubsecondTimer)
                .with_ansi(false)
                .with_target(true)
                .with_file(false)
                .with_line_number(false)
                .boxed(),
        };
    let ui_layer = ui_layer.with_filter(
        EnvFilter::builder()
            .with_default_directive(config.stdout_level.into())
            .from_env_lossy(),
    );

    // Per-level file layers
    let info_layer = make_file_layer(&log_dir, "info.json", |m| {
        m.level() == &tracing::Level::INFO
    }, &mut guards);
    let debug_layer = make_file_layer(&log_dir, "debug.json", |m| {
        m.level() == &tracing::Level::DEBUG
    }, &mut guards);
    let warn_layer = make_file_layer(&log_dir, "warn.json", |m| {
        m.level() == &tracing::Level::WARN
    }, &mut guards);
    let error_layer = make_file_layer(&log_dir, "error.json", |m| {
        m.level() == &tracing::Level::ERROR
    }, &mut guards);
    let trace_layer = make_file_layer(&log_dir, "trace.json", |m| {
        m.level() == &tracing::Level::TRACE
    }, &mut guards);

    // Combined log.json (all levels)
    let (all_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "log.json"),
    );
    guards.push(g);
    let all_layer = fmt::layer()
        .json()
        .with_writer(BoxMakeWriter::new(all_w))
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::DEBUG.into())
                .from_env_lossy(),
        );

    let subscriber = registry()
        .with(ui_layer)
        .with(info_layer)
        .with(debug_layer)
        .with(warn_layer)
        .with(error_layer)
        .with(trace_layer)
        .with(all_layer);

    tracing::subscriber::set_global_default(subscriber)?;
    tracing_log::LogTracer::init()?;
    tracing::info!(log_dir = %log_dir.display(), "logging initialized");

    Ok(LoggingGuard {
        _guards: guards,
        log_dir,
    })
}
