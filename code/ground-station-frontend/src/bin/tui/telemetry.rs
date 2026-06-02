use std::sync::Arc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use ground_station_frontend::backend::BackendClient;
use ground_station_frontend::state::AppState;

/// Render the Telemetry tab (Tab 1).
pub fn render_telemetry(frame: &mut Frame, area: Rect, state: &Arc<AppState<impl BackendClient>>) {
    let connected = state.ws_connected.load(std::sync::atomic::Ordering::Relaxed);

    let base_style = if connected {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    render_sensor_panels(frame, chunks[0], state, base_style, connected);
    render_recent_history(frame, chunks[1], state, base_style, connected);
    render_transitions(frame, chunks[2], state, base_style);
}

fn render_sensor_panels(
    frame: &mut Frame,
    area: Rect,
    state: &Arc<AppState<impl BackendClient>>,
    base_style: Style,
    connected: bool,
) {
    let record = state.latest_record.load();
    let lines = match record.as_ref() {
        Some(r) => format_sensor_lines(r, &base_style),
        None => vec![Line::from(Span::styled(
            "Waiting for telemetry...",
            base_style,
        ))],
    };

    let mut title = " Flight State ".to_string();
    if !connected {
        if let Some(last_seen) = state.last_record_time.lock().unwrap_or_else(|p| p.into_inner()).as_ref() {
            let ago = last_seen.elapsed().as_secs_f64();
            title = format!(" Flight State  STALE — last seen {ago:.0}s ago ");
        } else {
            title += " STALE";
        }
    }

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(base_style);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn format_sensor_lines(record: &proto::record::Record, _base_style: &Style) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    match record.payload() {
        proto::record::RecordData::Altimeter(data) => {
            let alt = data.altitude.value;
            let press = data.pressure.value;
            let temp_c = data.temperature.value - 273.15;
            lines.push(Line::from(vec![
                Span::styled("Altimeter:", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("  {alt:.1} m    ")),
                Span::styled("Pressure:", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(" {press:.0} Pa    ")),
                Span::styled("Temp:", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(" {temp_c:.1} C")),
            ]));
        }
        proto::record::RecordData::Gps(data) => {
            let lat = data.coordinates.latitude;
            let lon = data.coordinates.longitude;
            let alt = data.altitude.value;
            let sats = data.num_of_fix_satellites;
            lines.push(Line::from(vec![
                Span::styled("GPS:", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "  {lat:.4}, {lon:.4}    Alt: {alt:.1} m    Sats: {sats}"
                )),
            ]));
        }
        proto::record::RecordData::Imu(data) => {
            let ax = data.acceleration.x.value;
            let ay = data.acceleration.y.value;
            let az = data.acceleration.z.value;
            let gx = data.gyro.x.value;
            let gy = data.gyro.y.value;
            let gz = data.gyro.z.value;
            let mx = data.mag.x.value;
            let my = data.mag.y.value;
            let mz = data.mag.z.value;
            let temp_c = data.temperature.value - 273.15;
            lines.push(Line::from(vec![
                Span::styled(
                    "IMU:",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(
                    "  accel(X: {ax:.1} Y: {ay:.1} Z: {az:.1})"
                )),
            ]));
            lines.push(Line::from(vec![
                Span::raw(format!(
                    "       gyro(X: {gx:.1} Y: {gy:.1} Z: {gz:.1})"
                )),
            ]));
            lines.push(Line::from(vec![
                Span::raw(format!(
                    "       mag(X: {mx:.1} Y: {my:.1} Z: {mz:.1})  temp: {temp_c:.1} C"
                )),
            ]));
        }
        proto::record::RecordData::FlightState(fs) => {
            let (label, color) = flight_state_style(*fs);
            lines.push(Line::from(vec![
                Span::styled(
                    "Flight State:",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {label} "), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ]));
        }
        proto::record::RecordData::Event(_) => {
            lines.push(Line::from(Span::raw("Event record")));
        }
        proto::record::RecordData::Error(_) => {
            lines.push(Line::from(Span::styled(
                "Error record",
                Style::default().fg(Color::Red),
            )));
        }
    }

    lines
}

fn render_recent_history(
    frame: &mut Frame,
    area: Rect,
    state: &Arc<AppState<impl BackendClient>>,
    base_style: Style,
    connected: bool,
) {
    let history = state
        .altitude
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let snapshot = history.snapshot();
    let transitions = state
        .transitions
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let session_start = state
        .session_start_time
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    let mut hist_lines: Vec<Line> = Vec::new();

    let available = area.height.saturating_sub(2) as usize;
    let start = snapshot.len().saturating_sub(available);

    for (i, (instant, alt)) in snapshot.iter().enumerate().skip(start) {
        let t = session_start
            .map(|start| {
                let d = instant.duration_since(start);
                d.as_secs_f64()
            })
            .unwrap_or(0.0);

        let transition_marker = transitions.iter().find(|(tr_time, _)| {
            let tr_elapsed = session_start
                .map(|start| tr_time.duration_since(start).as_secs_f64())
                .unwrap_or(0.0);
            (t - tr_elapsed).abs() < 0.5
        });

        let marker = match transition_marker {
            Some((_, state)) => {
                let (label, color) = transition_label_style(*state);
                Some((i, label, color))
            }
            None => None,
        };

        let mut spans = vec![Span::styled(
            format!(" T+{t:.1}s  alt: {:>8.1}m", alt.value),
            base_style,
        )];

        if let Some((_, label, color)) = &marker {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!(" <- {label}"),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ));
        }

        hist_lines.push(Line::from(spans));
    }

    let title = if connected {
        " Recent History (last 30s) ".to_string()
    } else {
        let ago = state.last_record_time.lock().unwrap_or_else(|p| p.into_inner())
            .map(|t| format!(" — last seen {:.0}s ago", t.elapsed().as_secs_f64()))
            .unwrap_or_default();
        format!(" Recent History (last 30s) STALE{ago} ")
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(base_style);

    let paragraph = Paragraph::new(hist_lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_transitions(
    frame: &mut Frame,
    area: Rect,
    state: &Arc<AppState<impl BackendClient>>,
    base_style: Style,
) {
    let transitions = state
        .transitions
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let session_start = state
        .session_start_time
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    let mut spans: Vec<Span> = Vec::new();

    for (time, state) in transitions.iter() {
        let elapsed = session_start
            .map(|start| time.duration_since(start).as_secs_f64())
            .unwrap_or(0.0);
        let (label, color) = transition_label_style(*state);
        spans.push(Span::styled(
            format!("{label} T+{elapsed:.1}s "),
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" | "));
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            "No transitions yet",
            Style::default().fg(base_style.fg.unwrap_or(Color::White)),
        ));
    }

    let block = Block::default()
        .title(" Transitions ")
        .borders(Borders::ALL)
        .border_style(base_style);

    let paragraph = Paragraph::new(Line::from(spans))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

pub fn flight_state_style(state: proto::flight_state::FlightState) -> (&'static str, Color) {
    match state {
        proto::flight_state::FlightState::PreArmed => ("Pre-Armed", Color::Cyan),
        proto::flight_state::FlightState::Armed => ("Armed", Color::Yellow),
        proto::flight_state::FlightState::RecoveryActivated => ("Deploy", Color::Red),
        proto::flight_state::FlightState::Touchdown => ("Touchdown", Color::Green),
    }
}

fn transition_label_style(state: proto::flight_state::FlightState) -> (&'static str, Color) {
    match state {
        proto::flight_state::FlightState::PreArmed => ("PRE", Color::Cyan),
        proto::flight_state::FlightState::Armed => ("Arm", Color::Yellow),
        proto::flight_state::FlightState::RecoveryActivated => ("Dep", Color::Red),
        proto::flight_state::FlightState::Touchdown => ("TD", Color::Green),
    }
}
