use std::sync::Arc;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use ground_station_frontend::backend::BackendClient;
use ground_station_frontend::state::AppState;

/// Render the Logs tab (Tab 2) — placeholder for M3.2.
///
/// Backend does not emit `log` messages yet. This tab shows the
/// component color-coding reference table and a placeholder message.
/// When log_buffer has content (M3.6+), it renders log lines instead.
pub fn render_logs(frame: &mut Frame, area: Rect, state: &Arc<AppState<impl BackendClient>>) {
    let buffer = state
        .log_buffer
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    let mut lines: Vec<Line> = Vec::new();

    if buffer.is_empty() {
        // Placeholder content for M3.2.
        lines.push(Line::from(Span::styled(
            "Log streaming is inactive in M3.2.",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::raw(
            "When active (M3.6+), this tab streams logs from all system components:",
        )));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(vec![
            Span::styled("  [FC]    ", Style::default().fg(Color::Cyan)),
            Span::raw("Flight Computer — state transitions, sensor events"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  [SIM]   ", Style::default().fg(Color::Yellow)),
            Span::raw("Simulator — tick progress, scripted events"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  [GS-BE] ", Style::default().fg(Color::White)),
            Span::raw("Ground Station Backend — connection, storage, routing"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  [GS-FE] ", Style::default().fg(Color::Green)),
            Span::raw("Ground Station Frontend — WS health, command results"),
        ]));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "Each component is color-coded for quick scanning.",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Under HW deployment: FC logs are on-board only (WS stream carries",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "[GS-BE] and [GS-FE] logs only).",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Forward-compatible: render buffered log lines.
        for line in buffer.iter().rev() {
            lines.push(Line::from(Span::raw(line.clone())));
        }
    }

    let block = Block::default()
        .title(" Logs ")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
