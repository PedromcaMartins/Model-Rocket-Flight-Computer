use std::sync::Arc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use ground_station_frontend::backend::BackendClient;
use ground_station_frontend::state::AppState;

/// Render the Controls tab (Tab 3).
pub fn render_controls(frame: &mut Frame, area: Rect, state: &Arc<AppState<impl BackendClient>>) {
    let s = state.status.lock().unwrap_or_else(|p| p.into_inner());
    let last_cmd = state.last_cmd_result.lock().unwrap_or_else(|p| p.into_inner()).clone();
    let last_err = state.last_error.lock().unwrap_or_else(|p| p.into_inner()).clone();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Min(3),
        ])
        .split(area);

    render_commands(frame, chunks[0], s.fc_connected, &last_cmd);
    render_connection(frame, chunks[1], &last_err);
    render_keybinds(frame, chunks[2]);
}

// ---------------------------------------------------------------------------
// Command buttons
// ---------------------------------------------------------------------------

fn render_commands(frame: &mut Frame, area: Rect, connected: bool, last_cmd: &Option<String>) {
    let _ = frame;
    let mut lines = Vec::new();

    // Arm button.
    let arm_status = if connected {
        ""
    } else {
        "  (FC disconnected)"
    };
    lines.push(Line::from(vec![
        Span::styled("  [a] ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("Arm System{arm_status}")),
    ]));

    // Arm last result.
    if let Some(ref result) = *last_cmd
        && result.contains("Arm")
    {
        lines.push(Line::from(vec![
            Span::raw("       Last: "),
            Span::styled(
                result.clone(),
                Style::default().fg(if result.contains("OK") {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
        ]));
    }

    lines.push(Line::from(Span::raw("")));

    // Ignite button.
    let ignite_status = if connected {
        ""
    } else {
        "  (FC disconnected)"
    };
    lines.push(Line::from(vec![
        Span::styled("  [i] ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("Motor Ignition{ignite_status}")),
    ]));

    // Ignite last result.
    if let Some(ref result) = *last_cmd
        && result.contains("Ignite")
    {
        lines.push(Line::from(vec![
            Span::raw("       Last: "),
            Span::styled(
                result.clone(),
                Style::default().fg(if result.contains("OK") {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
        ]));
    }

    let block = Block::default()
        .title(" Commands ")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Connection status
// ---------------------------------------------------------------------------

fn render_connection(
    frame: &mut Frame,
    area: Rect,
    last_err: &Option<String>,
) {
    let mut lines = Vec::new();

    // Error display.
    if let Some(ref err) = *last_err {
        lines.push(Line::from(Span::styled(
            format!("  Error: {err}"),
            Style::default().fg(Color::Red),
        )));
    }

    lines.push(Line::from(Span::raw("")));
    let reconnect_secs = ground_station_frontend::config::Config::RECONNECT_INTERVAL.as_secs();
    lines.push(Line::from(Span::raw(format!("  Auto-reconnect: {reconnect_secs}s interval"))));

    let block = Block::default()
        .title(" Connection ")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Keybinds reference
// ---------------------------------------------------------------------------

fn render_keybinds(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            "  Keybinds",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::raw("  a = Arm System")),
        Line::from(Span::raw("  i = Motor Ignition")),
        Line::from(Span::raw("  q / Ctrl+C = Quit")),
        Line::from(Span::raw("")),
        Line::from(Span::raw("  Tab / Shift+Tab = Switch tabs")),
        Line::from(Span::raw("  1 = Telemetry  2 = Logs  3 = Controls")),
    ];

    let block = Block::default()
        .title(" Keybinds ")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
