use egui_sdl2_gl::egui::{self as egui};

const MAX_ZOOM: f32 = 20.0;
const MIN_ZOOM: f32 = 0.3;

pub struct Canvas {
    pan_offset: egui::Vec2, // Current pan offset
    zoom: f32, // Current zoom level
}

impl Canvas {
    pub fn new() -> Self {
        Canvas {
            pan_offset: egui::Vec2::ZERO,
            zoom: 1.0,
        }
    }

    pub fn jump_to(&mut self, x: f32, y: f32) {
        self.pan_offset = egui::Vec2::new(x, y);
    }
}
impl Default for Canvas {
    fn default() -> Self {
        Self {
            pan_offset: egui::Vec2::new(0.0, 0.0),
            zoom: 1.0, // Start with no zoom
        }
    }
}

impl Canvas {
    // Method to render the canvas and its contents
    pub fn draw(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Canvas background
            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::drag());
            let scroll_delta = ctx.input(|i| {
                i.scroll_delta
            });

            if response.hovered() {
                // Handle panning
                self.pan_offset -= response.drag_delta() / self.zoom;

                // Handle zooming
                let zoom_speed = 0.01;

                // Clamp the zoom level
                self.zoom *= (scroll_delta.y * zoom_speed).exp();
                self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
            }

            // Draw the grid
            self.draw_grid(&painter, response.rect);

            // Draw logic gates
            // for gate in &self.gates {
            //     self.draw_gate(&painter, gate);
            // }
        });
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        let grid_color = egui::Color32::from_gray(200); // Light gray for the grid lines
        let line_width = 0.2; // Width of the grid lines
    
        let grid_spacing = 20.0 * self.zoom; // Adjust grid spacing based on the zoom level
    
        // Calculate the first grid line before the visible area starts, adjusted by pan_offset
        let start_x = rect.min.x - (rect.min.x - self.pan_offset.x).rem_euclid(grid_spacing);
        let start_y = rect.min.y - (rect.min.y - self.pan_offset.y).rem_euclid(grid_spacing);
    
        // Calculate the number of lines to draw to cover the visible area
        let count_x = ((rect.width() + (rect.min.x - start_x).abs()) / grid_spacing).ceil() as i32;
        let count_y = ((rect.height() + (rect.min.y - start_y).abs()) / grid_spacing).ceil() as i32;
    
        // Draw vertical lines
        for i in 0..count_x {
            let x = start_x + i as f32 * grid_spacing;
            let start_point = egui::pos2(x, rect.min.y);
            let end_point = egui::pos2(x, rect.max.y);
            painter.line_segment([start_point, end_point], (line_width, grid_color));
        }
    
        // Draw horizontal lines
        for i in 0..count_y {
            let y = start_y + i as f32 * grid_spacing;
            let start_point = egui::pos2(rect.min.x, y);
            let end_point = egui::pos2(rect.max.x, y);
            painter.line_segment([start_point, end_point], (line_width, grid_color));
        }
    }
    
    

    // fn draw_gate(&self, painter: &egui::Painter, gate: &LogicGate) {
    //     // Draw the gate on the canvas
    // }
}
