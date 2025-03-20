use defmt_parser::Level;
use egui::{Color32, TextStyle};

use crate::LogMessage;

#[derive(PartialEq, Default)]
pub enum TerminalPresets {
    Raw,
    #[default]
    ProbeRS,
}

#[derive(Default)]
pub struct Terminal {
    config: TerminalPresets,
}

impl Terminal {
    pub fn ui(&mut self, ui: &mut egui::Ui, total_rows: usize, data: &[LogMessage]) {        
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);

        ui.horizontal(|ui| {
            ui.label("Presets");
            ui.radio_value(&mut self.config, TerminalPresets::Raw, "Raw");
            ui.radio_value(&mut self.config, TerminalPresets::ProbeRS, "Probe-rs");
        });

        ui.add_space(8.0);
        ui.separator();

        egui::ScrollArea::both()
            .max_height(f32::INFINITY)
            .max_width(f32::INFINITY)
            .auto_shrink(false)
            .stick_to_bottom(true)
            .show_rows(
                ui,
                row_height,
                total_rows,
                |ui, row_range| {
                    for row in row_range {
                        let log = &data[row];
                        match self.config {
                            TerminalPresets::Raw => {
                                ui.label(format!("{}\t {:?}", row + 1, log));
                            }
                            TerminalPresets::ProbeRS => {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label("[");
                                    ui.label(log.timestamp.as_str());
                                    if let Some(level) = log.level {
                                        match level {
                                            Level::Trace => ui.colored_label(Color32::LIGHT_GRAY, "TRACE"),
                                            Level::Debug => ui.colored_label(Color32::WHITE, "DEBUG"),
                                            Level::Info  => ui.colored_label(Color32::LIGHT_GREEN, "INFO"),
                                            Level::Warn  => ui.colored_label(Color32::LIGHT_YELLOW, "WARN"),
                                            Level::Error => ui.colored_label(Color32::LIGHT_RED, "ERROR"),
                                        };
                                    }
                                    if let Some(ref loc) = log.location {
                                        ui.label(format!("{:?}", loc));
                                    }
                                    ui.label("]");
                                    ui.label(&log.message);
                                });
                            }
                        }
                    }
                }
            );

        ui.ctx().request_repaint();
    }
}
