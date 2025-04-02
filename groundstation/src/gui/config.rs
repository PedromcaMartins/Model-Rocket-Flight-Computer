use std::path::PathBuf;

use egui::{Color32, ComboBox};
use tokio::sync::mpsc;

use crate::defmt_parser::Source;

#[derive(PartialEq, Eq)]
enum SelectedSource {
    Serial,
    None,
}

pub struct Config {
    /// The selected input source
    selected_input: SelectedSource,
    /// com port configurations
    com_port: Option<String>,
    baud_rate: String,
    /// Channel to send the selected source to the main thread
    tx: mpsc::Sender<Option<Source>>,
}

impl Config {
    pub fn new(tx: mpsc::Sender<Option<Source>>) -> Self {
        Self {
            selected_input: SelectedSource::None,
            com_port: None,
            baud_rate: "115200".to_string(),
            tx,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.serial_ui(ui);

        ui.separator();

        self.status_ui(ui);
    }

    pub fn serial_ui(&mut self, ui: &mut egui::Ui) {
        let ports = tokio_serial::available_ports().ok();

        ui.horizontal(|ui| {
            ComboBox::from_label("")
            .selected_text(self.com_port.as_deref().unwrap_or("Select COM Port"))
            .show_ui(ui, |ui| {
                if let Some(ports) = ports {
                    for port in ports {
                        ui.selectable_value(
                            &mut self.com_port, 
                            Some(port.port_name.clone()), 
                            &port.port_name
                        );
                    }
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Baud Rate: ");
                ui.add_sized([75.0, 15.0], egui::TextEdit::singleline(&mut self.baud_rate));
            });

            ui.add_space(8.0);

            if ui.button("Connect").on_hover_text("Connect to the COM port").clicked() && 
            self.connect_serial(ui).is_err() {
                    log::error!("TODO!");
            };
        });
    }

    pub fn status_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Status: ");

            match self.selected_input {
                SelectedSource::Serial if self.com_port.is_some() => {
                    ui.colored_label(Color32::DARK_GREEN, "Connected");
                    ui.label(format!("to {}", self.com_port.as_ref().unwrap()));
                },
                SelectedSource::None => {
                    ui.label("Disconnected");
                }
                _ => {
                    self.disconnect();
                }
            }

            ui.add_space(8.0);

            if self.selected_input != SelectedSource::None && 
            ui.button("Disconnect").clicked() {
                self.disconnect();
            }
        });
    }

    pub fn connect_serial(&mut self, _ui: &mut egui::Ui) -> Result<(), ()> {
        if let Some(port) = &self.com_port {
            let baud_rate = self.baud_rate.parse::<u32>().unwrap();
            let source = Source::serial(PathBuf::from(port), baud_rate).ok();
            match source {
                Some(ref s) => {
                    self.selected_input = SelectedSource::Serial;
                    log::info!("Selected COM port {:?} to connect", s);
                    self.tx.try_send(source).unwrap();
                },
                None => {
                    self.disconnect();
                    log::error!("Failed to connect COM port");
                    return Err(());
                }
            }
        } else {
            log::warn!("Tried to connect to a COM port, but none was selected");
            return Err(());
        }
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.com_port = None;
        self.selected_input = SelectedSource::None;
        self.tx.try_send(None).unwrap();
    }
}
