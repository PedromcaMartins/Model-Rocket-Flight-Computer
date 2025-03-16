use std::fmt::format;

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use tokio::{sync::mpsc, time::Instant};

use crate::Message;

pub struct MyApp {
    rx: mpsc::Receiver<Message>,
    data: Vec<Message>
}

impl MyApp {
    pub fn new(rx: mpsc::Receiver<Message>) -> Self {
        Self { rx, data: Vec::new() }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Receive Message updates
        while let Ok(point) = self.rx.try_recv() {
            self.data.push(point);
            if self.data.len() > 100 {
                self.data.remove(0); // Keep last 500 points for performance
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // display the messages in a scrollable area
            ui.heading("Telemetry Data");
            egui::ScrollArea::both().show(ui, |ui| {
                for message in &self.data {
                    ui.label(message);
                }
            });
        });

        ctx.request_repaint(); // Keep updating UI
    }
}

pub async fn simulated_telem(tx: mpsc::Sender<Message>) {
    let start_time = Instant::now();
    let mut value = 0;
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        // let time = start_time.elapsed().as_secs_f64();
        // value = (time * 2.0).sin(); // Simulated Message data (sine wave)
        value += 1;
        tx.send(format!("{}", value)).await.ok();
    }
}