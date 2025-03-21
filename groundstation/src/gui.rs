use eframe::egui;
use tokio::sync::mpsc;

use crate::{defmt_parser::{DefmtParser, Source}, LogMessage};

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
    #[default]
    Terminal,
    /// The configuration section
    Config,
    /// The graphs section
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
    /// The log messages
    data: Vec<LogMessage>,

    /// The current section
    section: Sections,
    /// The terminal section
    terminal: Terminal,
    /// The configuration section
    config: Config,
    /// The graphs section
    graphs: Graphs,
}

impl Default for GroundStation {
    fn default() -> Self {
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
            data: Vec::new(), 

            section: Default::default(),
            terminal: Default::default(),
            config: Config::new(tx_source),
            graphs: Default::default(),
        }
    }
}

impl GroundStation {
    /// update the data using messages from the receiver
    fn update_data(&mut self) {
        while let Ok(point) = self.rx_log_messages.try_recv() {
            self.data.push(point);
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
                    self.terminal.ui(ui, self.data.len(), &self.data);
                },
                Sections::Config => {
                    self.config.ui(ui);
                },
                Sections::Graphs => {
                    self.graphs.ui(ui);
                },
            }
        });

        ctx.request_repaint(); // Keep updating UI
    }
}
