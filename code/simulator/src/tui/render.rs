use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use proto::uom::si::{
    acceleration::meter_per_second_squared,
    length::meter,
    time::second,
    velocity::meter_per_second,
};

use super::actuators::render_actuators;
use super::logs::render_logs;
use super::TuiState;
use crate::config::Config;
use crate::physics::state::PhysicsState;
use crate::types::{ForceEvent, SimActuatorSnapshot};

pub(super) fn log_viewport_height(term_h: u16, maximized: bool) -> u16 {
    let outer = if maximized {
        term_h
    } else {
        term_h
            .saturating_sub(Config::PHYSICS_PANEL_HEIGHT)
            .saturating_sub(Config::EVENTS_PANEL_HEIGHT)
            .saturating_sub(Config::ACTUATOR_PANEL_HEIGHT)
    };
    outer.saturating_sub(2)
}

pub(super) fn render(
    f: &mut Frame,
    state: &TuiState,
    phys: &PhysicsState,
    forces: &[ForceEvent],
    act: &SimActuatorSnapshot,
    logs: &[String],
    fc_disconnected: bool,
) {
    let area = f.area();
    let main_area = if fc_disconnected {
        f.render_widget(
            Paragraph::new(" FC DISCONNECTED — q to quit ")
                .style(Style::default().fg(Color::White).bg(Color::Red)),
            Rect { x: area.x, y: area.y, width: area.width, height: 1 },
        );
        Rect { x: area.x, y: area.y + 1, width: area.width, height: area.height.saturating_sub(1) }
    } else {
        area
    };

    if state.maximized {
        render_logs(f, main_area, logs, state.scroll, true);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(Config::PHYSICS_PANEL_HEIGHT),
            Constraint::Length(Config::EVENTS_PANEL_HEIGHT),
            Constraint::Length(Config::ACTUATOR_PANEL_HEIGHT),
            Constraint::Min(Config::LOG_PANEL_MIN_HEIGHT),
        ])
        .split(main_area);

    let physics_text = format!(
        "Sim time  : {:.3} s\n\
         Altitude  : {:.2} m\n\
         Velocity  : {:.3} m/s\n\
         Accel     : {:.3} m/s²\n\
         Ignited   : {}\n\
         Deployed  : {}\n\
         TouchDown : {}",
        phys.time.get::<second>(),
        phys.altitude.get::<meter>(),
        phys.velocity.get::<meter_per_second>(),
        phys.acceleration.get::<meter_per_second_squared>(),
        phys.motor_ignited.map_or("no", |_| "YES"),
        phys.recovery_deployed.map_or("no", |_| "YES"),
        phys.touched_down.map_or("no", |_| "YES"),
    );

    let events_text = if forces.is_empty() {
        "  (none)".to_string()
    } else {
        let labels: Vec<String> = forces.iter().map(|e| format!("  {}", e)).collect();
        format!("Active forces:\n{}", labels.join("\n"))
    };

    let physics_block = Paragraph::new(physics_text)
        .block(Block::default().title(" Physics State ").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(physics_block, chunks[0]);

    let events_block = Paragraph::new(events_text)
        .block(Block::default().title(" Events ").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    f.render_widget(events_block, chunks[1]);

    render_actuators(f, chunks[2], act);

    render_logs(f, chunks[3], logs, state.scroll, false);
}
