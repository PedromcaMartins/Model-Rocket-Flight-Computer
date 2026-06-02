//! TUI event loop and command-spawning helpers.

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use scopeguard::defer;
use tracing::info;

use super::ActiveTab;
use ground_station_frontend::backend::{BackendClient, WsBackend};
use ground_station_frontend::config::Config;
use ground_station_frontend::state::AppState;

// ---------------------------------------------------------------------------
// TUI event loop (blocking)
// ---------------------------------------------------------------------------

pub fn run_tui(state: &Arc<AppState<WsBackend>>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;

    defer! {
        let _ = std::io::stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;

    let mut active_tab = ActiveTab::Telemetry;
    let tick = Duration::from_millis(1000 / Config::TUI_FPS as u64);

    loop {
        terminal.draw(|frame| {
            super::render::render_layout(frame, state, active_tab);
        })?;

        if event::poll(tick)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match key.code {
                        // Quit keys.
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }

                        // Tab switching.
                        KeyCode::Tab => active_tab = active_tab.next(),
                        KeyCode::BackTab => active_tab = active_tab.prev(),
                        KeyCode::Char('1') => active_tab = ActiveTab::Telemetry,
                        KeyCode::Char('2') => active_tab = ActiveTab::Logs,
                        KeyCode::Char('3') => active_tab = ActiveTab::Controls,

                        // Arm.
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            info!("Arm command issued");
                            spawn_arm(state.clone());
                        }

                        // Ignite.
                        KeyCode::Char('i') | KeyCode::Char('I') => {
                            info!("Ignite command issued");
                            spawn_ignite(state.clone());
                        }

                        _ => {}
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}

fn spawn_cmd(
    state: Arc<AppState<WsBackend>>,
    name: &'static str,
    fut: impl std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
) {
    tokio::spawn(async move {
        match fut.await {
            Ok(()) => {
                *state.last_cmd_result.lock().unwrap_or_else(|p| p.into_inner()) =
                    Some(format!("{name} OK"));
            }
            Err(e) => {
                *state.last_cmd_result.lock().unwrap_or_else(|p| p.into_inner()) =
                    Some(format!("{name} failed: {e}"));
            }
        }
    });
}

fn spawn_arm(state: Arc<AppState<WsBackend>>) {
    let cmd_state = state.clone();
    spawn_cmd(state, "Arm", async move { cmd_state.backend.arm().await });
}

fn spawn_ignite(state: Arc<AppState<WsBackend>>) {
    let cmd_state = state.clone();
    spawn_cmd(state, "Ignite", async move { cmd_state.backend.ignite().await });
}
