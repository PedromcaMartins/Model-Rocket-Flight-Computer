use egui_plotter::{EguiBackend, DEFAULT_MOVE_SCALE, DEFAULT_SCROLL_SCALE};
use telemetry::{AltimeterMessage, GpsMessage, ImuMessage};
use plotters::prelude::*;

pub struct MouseData {
    chart_pitch: f32,
    chart_yaw: f32,
    chart_scale: f32,
    chart_pitch_vel: f32,
    chart_yaw_vel: f32,
}

impl Default for MouseData {
    fn default() -> Self {
        Self {
            chart_pitch: 0.5,
            chart_yaw: 0.5,
            chart_scale: 0.5,
            chart_pitch_vel: 0.0,
            chart_yaw_vel: 0.0,
        }
    }
}

impl MouseData {
    pub fn update_mouse_data(&mut self, ui: &mut egui::Ui) {
        let panel_rect = ui.max_rect(); // Get the bounding rectangle of the panel

        let (pitch_delta, yaw_delta, scale_delta) = ui.input(|input| {
            let pointer = &input.pointer;
            let delta = pointer.delta();

            // Check if the cursor is inside the panel
            let is_inside_panel = panel_rect.contains(pointer.interact_pos().unwrap_or_default());

            let (pitch_delta, yaw_delta) = match pointer.primary_down() && is_inside_panel {
                true => (delta.y * DEFAULT_MOVE_SCALE, -delta.x * DEFAULT_MOVE_SCALE),
                false => (self.chart_pitch_vel * DEFAULT_MOVE_SCALE, self.chart_yaw_vel * DEFAULT_MOVE_SCALE),
            };

            let scale_delta = if is_inside_panel { input.raw_scroll_delta.y * DEFAULT_SCROLL_SCALE } else { 0.0 };

            (pitch_delta, yaw_delta, scale_delta)
        });

        self.chart_pitch_vel = pitch_delta;
        self.chart_yaw_vel = yaw_delta;

        self.chart_pitch += self.chart_pitch_vel;
        self.chart_yaw += self.chart_yaw_vel;
        self.chart_scale += scale_delta;
    }
}

#[derive(Default)]
pub struct PositionChart {
    mouse_data: MouseData,
}

impl PositionChart {
    pub fn plot(
        &mut self, 
        ui: &mut egui::Ui, 
        altimeter_data: &[AltimeterMessage], 
        gps_data: &[GpsMessage],
        imu_data: &[ImuMessage],
    ) {
        let mut positions  = gps_data
            .iter()
            .zip(altimeter_data.iter())
            .rev()
            .take(100)
            .map(|(gps, altimeter)| {
                let lat = gps.latitude;
                let lon = gps.longitude;
                let alt = altimeter.altitude as f64;

                (lat, lon, alt)
            });

        let mut orientations = imu_data
            .iter()
            .rev()
            .take(100)
            .map(|imu| {
                let roll = imu.euler_angles[0];
                let pitch = imu.euler_angles[1];
                let yaw = imu.euler_angles[2];

                (yaw, pitch, roll)
            });

        let current_position = positions.next();
        let current_orientation = orientations.next();

        self.mouse_data.update_mouse_data(ui);

        // Next plot everything
        let root = EguiBackend::new(ui).into_drawing_area();

        root.fill(&WHITE).unwrap();

        let axis = (-3.0..3.0).step(0.1);

        // create a 3d chart
        let mut chart = ChartBuilder::on(&root)
            .caption("Position in space", (FontFamily::SansSerif, 20))
            .build_cartesian_3d(axis.clone(), axis.clone(), axis)
            .unwrap();

        // use mouse data to set projection
        chart.with_projection(|mut pb| {
            pb.yaw = self.mouse_data.chart_yaw as f64;
            pb.pitch = self.mouse_data.chart_pitch as f64;
            pb.scale = self.mouse_data.chart_scale as f64;
            pb.into_matrix()
        });

        // display the axes
        chart
            .configure_axes()
            .light_grid_style(BLACK.mix(0.15))
            .max_light_lines(3)
            .draw()
            .unwrap();

        // draw the trajectory from last positions
        chart
            .draw_series(LineSeries::new(
                positions,
                &BLACK,
            ))
            .unwrap()
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK));

        root.present().unwrap();
    }
}
