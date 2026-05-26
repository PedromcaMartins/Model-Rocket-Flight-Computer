use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use proto::actuator_data::LedStatus;

use crate::types::SimActuatorSnapshot;

pub(super) fn render_actuators(f: &mut Frame, area: Rect, act: &SimActuatorSnapshot) {
    let block = Block::default()
        .title(" Actuators ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let leds: [(&str, Color, LedStatus); 8] = [
        ("Postcard", Color::Red, act.postcard_led),
        ("Altimeter", Color::Red, act.altimeter_led),
        ("GPS", Color::Red, act.gps_led),
        ("IMU", Color::Red, act.imu_led),
        ("Arm", Color::Green, act.arm_led),
        ("File System", Color::Red, act.file_system_led),
        ("Deployment", Color::Rgb(255, 165, 0), act.deployment_led),
        ("Ground Station", Color::Red, act.ground_station_led),
    ];

    let cells = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 8); 8])
        .split(inner);

    for (i, (label, color, status)) in leds.iter().enumerate() {
        let cell = cells[i];
        let led_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(1)])
            .split(cell);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));
        let inner = block.inner(led_chunks[0]);
        f.render_widget(block, led_chunks[0]);

        if *status == LedStatus::On {
            f.render_widget(Paragraph::new("").style(Style::default().bg(*color)), inner);
        }
        f.render_widget(
            Paragraph::new(*label)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
            led_chunks[1],
        );
    }
}
