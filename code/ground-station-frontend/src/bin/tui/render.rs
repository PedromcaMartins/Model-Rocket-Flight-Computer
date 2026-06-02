use std::sync::Arc;

use chrono::Local;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use ground_station_frontend::backend::BackendClient;
use ground_station_frontend::state::AppState;

use super::ActiveTab;

/// Top-level render function: status bar + active tab content.
pub fn render_layout(frame: &mut Frame, state: &Arc<AppState<impl BackendClient>>, active_tab: ActiveTab) {
    let area = frame.area();

    // Split into status bar (1 row) + content area.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    render_status_bar(frame, chunks[0], state, active_tab);
    render_active_tab(frame, chunks[1], state, active_tab);
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

fn render_status_bar(frame: &mut Frame, area: Rect, state: &Arc<AppState<impl BackendClient>>, active_tab: ActiveTab) {
    let ws_ok = state.ws_connected.load(std::sync::atomic::Ordering::Relaxed);
    let ping = state.ping.load(std::sync::atomic::Ordering::Relaxed);
    let status = state.status.lock().unwrap_or_else(|p| p.into_inner());

    let text_color = if ws_ok {
           if ping { Color::Green } else { Color::DarkGray }
       } else {
           Color::Red
       };
    let text_style = Style::default().fg(text_color);

    let mut spans = vec![
        Span::styled("●", text_style.add_modifier(Modifier::BOLD)),
    ];

    if ws_ok {
        spans.push(Span::styled(" Connected", text_style));
        spans.push(Span::raw(" | "));
        if let Some(latency) = status.latency {
            spans.push(Span::styled(format!("Ping: {} ms", latency.as_millis()), Style::default().fg(Color::Green)));
        } else {
            spans.push(Span::styled("Ping: - ms", Style::default().fg(Color::Red)));
        }
        spans.push(Span::raw(" | "));
    } else {
        spans.push(Span::styled(" DISCONNECTED | ", text_style));
    }

    spans.push(Span::raw(format!("Records: {} | ", status.record_count)));
    let session = status
        .session_start
        .with_timezone(&Local)
        .format("%H:%M:%S")
        .to_string();
    spans.push(Span::raw(format!("Session: {} | ", session)));

    // Tab switcher.
    for (i, tab) in ["Telemetry", "Logs", "Controls"].iter().enumerate() {
        let is_active = match active_tab {
            ActiveTab::Telemetry => i == 0,
            ActiveTab::Logs => i == 1,
            ActiveTab::Controls => i == 2,
        };
        let label = format!("[{}] {}", i + 1, tab);
        if is_active {
            spans.push(Span::styled(
                format!(" {label} "),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::raw(format!(" {label} ")));
        }
    }

    spans.push(Span::raw(" | [q] quit"));

    let status_line = Line::from(spans);
    frame.render_widget(
        Paragraph::new(status_line)
            .block(Block::default().borders(Borders::NONE)),
        area,
    );
}

// ---------------------------------------------------------------------------
// Tab dispatch
// ---------------------------------------------------------------------------

fn render_active_tab(frame: &mut Frame, area: Rect, state: &Arc<AppState<impl BackendClient>>, active_tab: ActiveTab) {
    match active_tab {
        ActiveTab::Telemetry => super::telemetry::render_telemetry(frame, area, state),
        ActiveTab::Logs => super::logs::render_logs(frame, area, state),
        ActiveTab::Controls => super::controls::render_controls(frame, area, state),
    }
}
