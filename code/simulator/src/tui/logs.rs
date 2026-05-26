use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

fn level_color(token: &str) -> Color {
    match token {
        "ERROR" => Color::Red,
        "WARN" => Color::Yellow,
        "INFO" => Color::Green,
        "DEBUG" => Color::Cyan,
        "TRACE" => Color::Gray,
        _ => Color::White,
    }
}

const DIM: Color = Color::Rgb(0x66, 0x66, 0x66);

fn render_log_line(line: &str) -> Line<'_> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let level_idx = tokens.iter().position(|t| matches!(*t, "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE"));

    let Some(level_idx) = level_idx else {
        return Line::from(Span::raw(line));
    };

    let mut spans = vec![
        Span::styled(tokens[0], Style::default().fg(DIM)),
        Span::raw(" "),
        Span::styled(tokens[level_idx], Style::default().fg(level_color(tokens[level_idx]))),
    ];

    let rest = &tokens[level_idx + 1..];
    let meta_count = rest.iter().take_while(|t| t.ends_with(':')).count();

    for &token in &rest[..meta_count] {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(token, Style::default().fg(DIM)));
    }

    for &token in &rest[meta_count..] {
        spans.push(Span::raw(" "));
        spans.push(Span::raw(token));
    }

    Line::from(spans)
}

pub(super) fn render_logs(f: &mut Frame, area: Rect, logs: &[String], scroll: usize, maximized: bool) {
    let inner_h = area.height.saturating_sub(2) as usize;
    let max_scroll = logs.len().saturating_sub(inner_h);
    let scroll = scroll.min(max_scroll);
    let top = (max_scroll - scroll) as u16;

    let title = if maximized {
        " Logs [INFO+]  m/Tab: panel · scroll with mouse · q quit "
    } else {
        " Logs [INFO+]  m/Tab: maximize · scroll with mouse · q quit "
    };

    let lines: Vec<Line> = logs.iter().map(|log| render_log_line(log)).collect();
    let widget = Paragraph::new(Text::from(lines))
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .scroll((top, 0));
    f.render_widget(widget, area);
}
