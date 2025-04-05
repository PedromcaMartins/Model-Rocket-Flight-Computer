use egui::{Color32, Stroke};
use egui_plot::{Line, Plot};
use telemetry::{AltimeterMessage, GpsMessage, ImuMessage};

mod position;
use position::PositionChart;

#[derive(Default)]
pub struct Graphs {
    position_chart: PositionChart,
}

impl Graphs {
    pub fn ui(
        &mut self, 
        ui: &mut egui::Ui, 
        imu_data: &[ImuMessage], 
        altimeter_data: &[AltimeterMessage], 
        gps_data: &[GpsMessage]
    ) {
        let window_width = ui.available_width();

        egui::TopBottomPanel::top("top_panel")
        .resizable(false)
        .min_height(100.0)
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Real-Time Data");
            });
        });

        egui::SidePanel::left("left_panel")
        .resizable(false)
        .min_width(200.0)
        .show_inside(ui, |ui| {
            let panel_width = f32::max(window_width * 0.18, 200.0);
            let panel_height = ui.available_height();
            let chart_height = panel_height / 3.0; // Divide height between 3 charts

            let (points_x, points_y, points_z) = imu_data
                .iter()
                .rev()
                .take(200)
                .map(|data| ((data.timestamp as f64 / 1_000_000_f64), data.acceleration))
                .map(|(ts, [x, y, z])| ([ts, x as f64], [ts, y as f64], [ts, z as f64]))
                .collect::<(Vec<_>, Vec<_>, Vec<_>)>();

            for (points, label, color) in [
                (points_x, "Acceleration X", Color32::LIGHT_RED),
                (points_y, "Acceleration Y", Color32::LIGHT_GREEN),
                (points_z, "Acceleration Z", Color32::LIGHT_BLUE)
            ] {
                ui.allocate_ui([panel_width, chart_height].into(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(label);
                        ui.add_space(4.0);

                        Plot::new(label)
                        .legend(Default::default())
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(points)
                                .stroke(Stroke::new(2.0, color))
                            );
                        });
                    });
                });
            }
        });

        egui::SidePanel::right("right_panel")
        .resizable(false)
        .min_width(200.0)
        .show_inside(ui, |ui| {
            let panel_width = f32::max(window_width * 0.18, 200.0);
            let panel_height = ui.available_height();
            let chart_height = panel_height / 3.0; // Divide height between 3 charts

            let (points_x, points_y, points_z) = imu_data
                .iter()
                .rev()
                .take(200)
                .map(|data| ((data.timestamp as f64 / 1_000_000_f64), data.euler_angles))
                .map(|(ts, [x, y, z])| ([ts, x as f64], [ts, y as f64], [ts, z as f64]))
                .collect::<(Vec<_>, Vec<_>, Vec<_>)>();

            for (points, label, color) in [
                (points_x, "Roll", Color32::LIGHT_RED),
                (points_y, "Pitch", Color32::LIGHT_GREEN),
                (points_z, "Yaw", Color32::LIGHT_BLUE)
            ] {
                ui.allocate_ui([panel_width, chart_height].into(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(label);
                        ui.add_space(4.0);

                        Plot::new(label)
                        .legend(Default::default())
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(points)
                                .stroke(Stroke::new(2.0, color))
                            );
                        });
                    });
                });
            }
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.centered_and_justified(|ui| {
                self.position_chart.plot(ui, altimeter_data, gps_data, imu_data);
            });
        });
    }
}
