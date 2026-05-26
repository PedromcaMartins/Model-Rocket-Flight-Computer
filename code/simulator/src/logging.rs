use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Local;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::{FilterFn, LevelFilter};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, registry, EnvFilter};

use crate::config::Config;
use buffer_writer::{BufferMakeWriter, LoggingGuard};

/// Shared ring buffer of recent log lines, rendered by the TUI log panel.
/// Replaces the stdout layer — stdout can't coexist with a fullscreen TUI.
pub static LOG_BUFFER: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // The TUI owns the alternate screen + raw mode; restore the terminal
        // first so the panic message is readable and the shell isn't wrecked.
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen
        );
        tracing::error!(%info, "process panicked");
        default_hook(info);
    }));
}

pub fn init_tracing() -> anyhow::Result<LoggingGuard> {
    LOG_BUFFER
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .reserve(Config::LOG_BUFFER_CAPACITY);

    let log_dir = log_dir();
    fs::create_dir_all(&log_dir)?;

    let mut guards = Vec::new();

    // TUI log panel: strictly INFO and above, regardless of RUST_LOG.
    let tui_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(BufferMakeWriter)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(LevelFilter::INFO);

    let (info_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "info.json"),
    );
    guards.push(g);
    let info_layer = fmt::layer()
        .json()
        .with_writer(info_w)
        .with_filter(FilterFn::new(|m| m.level() == &tracing::Level::INFO));

    let (debug_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "debug.json"),
    );
    guards.push(g);
    let debug_layer = fmt::layer()
        .json()
        .with_writer(debug_w)
        .with_filter(FilterFn::new(|m| m.level() == &tracing::Level::DEBUG));

    let (warn_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "warn.json"),
    );
    guards.push(g);
    let warn_layer = fmt::layer()
        .json()
        .with_writer(warn_w)
        .with_filter(FilterFn::new(|m| m.level() == &tracing::Level::WARN));

    let (error_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "error.json"),
    );
    guards.push(g);
    let error_layer = fmt::layer()
        .json()
        .with_writer(error_w)
        .with_filter(FilterFn::new(|m| m.level() == &tracing::Level::ERROR));

    let (trace_w, g) = tracing_appender::non_blocking(
        tracing_appender::rolling::never(&log_dir, "trace.json"),
    );
    guards.push(g);
    let trace_layer = fmt::layer()
        .json()
        .with_writer(trace_w)
        .with_filter(FilterFn::new(|m| m.level() == &tracing::Level::TRACE));

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
        .with(tui_layer)
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
    let ts = Local::now().format(Config::LOG_TIMESTAMP_FORMAT).to_string();
    PathBuf::from(Config::LOG_ROOT_DIR).join(ts)
}

pub (super) mod buffer_writer {
    use super::*;

    pub struct LoggingGuard {
        pub _guards: Vec<WorkerGuard>,
    }

    pub struct BufferWriter;

    impl std::io::Write for BufferWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if let Ok(text) = std::str::from_utf8(bytes) {
                // Recover from a poisoned guard so a panic on a thread that
                // held the lock can't cascade through the panic hook's
                // re-entry into tracing.
                let mut guard = LOG_BUFFER.lock().unwrap_or_else(|p| p.into_inner());
                for line in text.lines() {
                    if !line.is_empty() {
                        guard.push_back(line.to_string());
                    }
                }
                while guard.len() > Config::LOG_BUFFER_CAPACITY {
                    guard.pop_front();
                }
            }
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[derive(Clone)]
    pub struct BufferMakeWriter;

    impl<'a> MakeWriter<'a> for BufferMakeWriter {
        type Writer = BufferWriter;
        fn make_writer(&'a self) -> Self::Writer {
            BufferWriter
        }
    }
}
