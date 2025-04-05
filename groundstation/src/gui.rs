use eframe::egui;
use telemetry::{AltimeterMessage, GpsMessage, ImuMessage};
use tokio::sync::mpsc;

use crate::parser::{DefmtParser, LogMessage, MessageType, Source};

mod terminal;
use terminal::Terminal;

mod config;
use config::Config;

mod graphs;
use graphs::Graphs;

#[derive(PartialEq, Default)]
enum Sections {
    // Home,
    /// The terminal section
    Terminal,
    /// The configuration section
    Config,
    /// The graphs section
    #[default]
    Graphs,
}

/// The Ground Station
/// 
/// Keeps track of the log messages and the GUI state
pub struct GroundStation {
    /// The receiver for the log messages
    rx_log_messages: mpsc::Receiver<LogMessage>,
    /// The sender for the log messages
    tx_log_messages: mpsc::Sender<LogMessage>,

    /// The log messages from strings
    string_messages: Vec<LogMessage>,
    /// The log messages from imu
    imu_messages: Vec<ImuMessage>,
    /// The log messages from altimeter
    altimeter_messages: Vec<AltimeterMessage>,
    /// The log messages from gps
    gps_messages: Vec<GpsMessage>,

    /// The current section
    section: Sections,
    /// The terminal section
    terminal: Terminal,
    /// The configuration section
    config: Config,
    /// The graphs section
    graphs: Graphs,
}

impl GroundStation {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Disable feathering as it causes artifacts with plotters
        let context = &cc.egui_ctx;
        context.tessellation_options_mut(|tess_options| {
            tess_options.feathering = false;
        });
        context.set_pixels_per_point(0.8); // Disable horizontal / vertical bands

        let (tx_log_messages, rx_log_messages) = mpsc::channel::<LogMessage>(100);
        let (tx_source, rx_source) = mpsc::channel::<Option<Source>>(1);

        let tx = tx_log_messages.clone();
        tokio::spawn(async move {
            let mut defmt_parser = DefmtParser::new(tx, rx_source).await.unwrap();
            defmt_parser.run().await.unwrap();
        });

        Self {
            rx_log_messages, 
            tx_log_messages,

            string_messages: Default::default(), 
            imu_messages: Default::default(),
            altimeter_messages: Default::default(),
            gps_messages: Default::default(),

            section: Default::default(),
            terminal: Default::default(),
            config: Config::new(tx_source),
            graphs: Default::default(),
        }
    }

    /// update the data using messages from the receiver
    fn update_data(&mut self) {
        while let Ok(log) = self.rx_log_messages.try_recv() {
            match log.message {
                MessageType::String(_) => {
                    self.string_messages.push(log);
                },
                MessageType::ImuMessage(message) => {
                    self.imu_messages.push(message);
                },
                MessageType::AltimeterMessage(message) => {
                    self.altimeter_messages.push(message);
                },
                MessageType::GpsMessage(message) => {
                    self.gps_messages.push(message);
                },
            }
        }
    }

    pub fn clone_tx(&self) -> mpsc::Sender<LogMessage> {
        self.tx_log_messages.clone()
    }
}

impl eframe::App for GroundStation {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_data();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.section, Sections::Terminal, "Terminal");
                ui.selectable_value(&mut self.section, Sections::Config, "Config");
                ui.selectable_value(&mut self.section, Sections::Graphs, "Graphs");
            });

            ui.separator();

            match self.section {
                Sections::Terminal => {
                    self.terminal.ui(ui, &self.string_messages);
                },
                Sections::Config => {
                    self.config.ui(ui);
                },
                Sections::Graphs => {
                    self.graphs.ui(ui, &self.imu_messages, &self.altimeter_messages, &self.gps_messages);
                },
            }
        });

        ctx.request_repaint(); // Keep updating UI
    }
}
