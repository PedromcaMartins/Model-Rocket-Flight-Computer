use eframe::egui;
use tokio::{sync::mpsc, time::Instant};

use crate::Message;

#[derive(PartialEq)]
enum Sections {
    // Home,
    Terminal,
    // Graphs,
}

pub struct GroundStation {
    rx: mpsc::Receiver<Message>,
    data: Vec<Message>,
    section: Sections,
}

impl GroundStation {
    pub fn new(rx: mpsc::Receiver<Message>) -> Self {
        Self { rx, data: Vec::new(), section: Sections::Terminal }
    }

    fn update_data(&mut self) {
        while let Ok(point) = self.rx.try_recv() {
            self.data.push(point);
            if self.data.len() > 100 {
                self.data.remove(0); // Keep last 500 points for performance
            }
        }
    }

    fn terminal(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both()
            .max_height(f32::INFINITY)
            .max_width(f32::INFINITY)
            .auto_shrink(false)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for (time, value) in &self.data {
                    ui.label(format!("Time: {}, Value: {}", time, value));
                }
            });
    }
}

impl eframe::App for GroundStation {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_data();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.selectable_value(&mut self.section, Sections::Terminal, "Terminal");
            ui.separator();
            match self.section {
                Sections::Terminal => {
                    self.terminal(ui);
                }
            }
        });

        ctx.request_repaint(); // Keep updating UI
    }
}

pub async fn simulated_telem(tx: mpsc::Sender<Message>) {
    let start_time = Instant::now();
    let mut value;
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        let time = start_time.elapsed().as_millis() as u64;
        value = (time as f64 * 2.0).sin(); // Simulated telemetry data (sine wave)
        tx.send((time, value)).await.ok();
    }
}