use std::path::PathBuf;

use egui::{Color32, ComboBox};
use tokio::sync::mpsc;

use crate::defmt_parser::Source;

#[derive(PartialEq, Default)]
pub enum InputConfig {
    #[default]
    Serial,
}

pub struct Config {
    // input config
    com_port: Option<String>,
    tx: mpsc::Sender<Option<Source>>,
}

impl Config {
    pub fn new(tx: mpsc::Sender<Option<Source>>) -> Self {
        Self {
            com_port: None,
            tx,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let ports = tokio_serial::available_ports().ok();

            ComboBox::from_label("COM Port")
            .selected_text(self.com_port.as_deref().unwrap_or("Select COM Port"))
            .show_ui(ui, |ui| {
                if let Some(ports) = ports {
                    for port in ports {
                        if ui.selectable_value(
                            &mut self.com_port, 
                            Some(port.port_name.clone()), 
                            &port.port_name
                        ).changed() {
                            let source = Source::serial(PathBuf::from(&port.port_name), 115200).ok();
                            if source.is_none() {
                                self.com_port = None;
                                ui.colored_label(Color32::DARK_RED, "Failed to connect");
                            }
                            self.tx.try_send(source).unwrap();
                        }
                    }
                }
            });

            ui.add_space(8.0);

            match &self.com_port {
                Some(port) => {
                    ui.colored_label(Color32::DARK_GREEN, "Connected");
                    ui.label(format!("to {}", port));
                    ui.add_space(8.0);
                    if ui.button("Disconnect").on_hover_text("Disconnect from the COM port").clicked() {
                        self.com_port = None;
                        self.tx.try_send(None).unwrap();
                    };
                }
                None => {
                    ui.label("Disconnected");
                }
            }
        });
    }
}
