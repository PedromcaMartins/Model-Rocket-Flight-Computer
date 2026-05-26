use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use arc_swap::ArcSwap;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use scopeguard::defer;
use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::logging;
use crate::physics::state::PhysicsState;
use crate::types::{ActiveForceEvent, SimActuatorSnapshot};

pub(crate) mod render;
pub(crate) mod actuators;
pub(crate) mod logs;

use self::render::render;

#[derive(Default)]
pub(crate) struct TuiState {
    scroll: usize,
    maximized: bool,
}

pub async fn run_tui(
    physics_state_rx: tokio::sync::watch::Receiver<PhysicsState>,
    active_forces: Arc<ActiveForceEvent>,
    actuator_snapshot: Arc<ArcSwap<SimActuatorSnapshot>>,
    cancel: CancellationToken,
    tui_cancel: CancellationToken,
) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        tui_blocking(physics_state_rx, active_forces, actuator_snapshot, cancel, tui_cancel)
    })
    .await
    .context("TUI blocking task panicked")?
}

fn tui_blocking(
    physics_state_rx: tokio::sync::watch::Receiver<PhysicsState>,
    active_forces: Arc<ActiveForceEvent>,
    actuator_snapshot: Arc<ArcSwap<SimActuatorSnapshot>>,
    cancel: CancellationToken,
    tui_cancel: CancellationToken,
) -> anyhow::Result<()> {
    // Initialize TUI
    crossterm::terminal::enable_raw_mode().with_context(|| "failed to enable raw mode")?;
    defer! { let _ = crossterm::terminal::disable_raw_mode(); }

    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture).with_context(|| "failed to enter alternate screen")?;
    defer! { let _ = crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen, crossterm::event::DisableMouseCapture); }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).with_context(|| "failed to create terminal")?;

    let mut state = TuiState::default();

    loop {
        if tui_cancel.is_cancelled() {
            anyhow::bail!("TUI quit");
        }

        // Poll for events (keyboard, mouse) and handle them
        if matches!(event::poll(Duration::from_secs(1) / Config::TUI_REFRESH_RATE as u32), Ok(true))
            && let Ok(event) = event::read()
        {
            match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            tui_cancel.cancel();
                            anyhow::bail!("User requested shutdown");
                        }
                        KeyCode::Char('c') if ctrl => {
                            tui_cancel.cancel();
                            anyhow::bail!("User requested shutdown");
                        }
                        KeyCode::Char('m') | KeyCode::Tab => {
                            state.maximized = !state.maximized;
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => {
                        state.scroll = state.scroll.saturating_sub(3);
                    }
                    MouseEventKind::ScrollUp => {
                        state.scroll = state.scroll.saturating_add(3);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let fc_disconnected = cancel.is_cancelled();
        let phys = physics_state_rx.borrow().clone();
        let forces = active_forces.load();
        let act = actuator_snapshot.load();
        let logs: Vec<String> = {
            let guard = logging::LOG_BUFFER.lock().unwrap_or_else(|p| p.into_inner());
            guard.iter().cloned().collect()
        };

        let term_h = terminal.size().map(|s| s.height).unwrap_or(24);
        let log_inner_h = render::log_viewport_height(term_h, state.maximized) as usize;
        let max_scroll = logs.len().saturating_sub(log_inner_h);
        state.scroll = state.scroll.min(max_scroll);

        let _ = terminal.draw(|f| render(f, &state, &phys, &forces, &act, &logs, fc_disconnected));
    }
}
