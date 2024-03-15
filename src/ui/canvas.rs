use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use egui_sdl2_gl::egui::{self as egui};
use crate::{ui::drawable_gate::DrawableGate};

use super::{connection::DrawableConnections, gate_list::GhostGate};

const MAX_ZOOM: f32 = 20.0;
const MIN_ZOOM: f32 = 0.3;
pub const GRID_SPACING: f32 = 20.0;

pub struct Canvas {
    pan_offset: egui::Vec2, // Current pan offset
    zoom: f32, // Current zoom level
    gates: Vec<Rc<DrawableGate>>, // List of gates on the canvas
    to_spawn: Option<GhostGate>,
    connections: Vec<DrawableConnections>,
    to_connect: Option<DrawableConnections>,
}

impl Canvas {
    pub fn new() -> Self {
        Canvas {
            pan_offset: egui::Vec2::ZERO,
            zoom: 1.0,
            gates: vec![],
            to_spawn: None,
            connections: vec![],
            to_connect: None,
        }
    }

    pub fn get_pan_offset(&self) -> egui::Vec2 {
        self.pan_offset
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn add_gate(&mut self, gate: DrawableGate) {
        self.gates.push(Rc::new(gate));
    }

    pub fn remove_selected(&mut self) {
        self.gates.retain(|gate| !gate.selected);
    }

    pub fn unselect_all(&mut self) {
        for gate in &mut self.gates {
            gate.selected = false;
        }
    }

    pub fn add_to_spawn(&mut self, gate: GhostGate) {
        self.to_spawn = Some(gate);
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
            gates: Vec::new(),
            to_spawn: None,
            connections: Vec::new(),
            to_connect: None,
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

            if ctx.input(|k| {k.key_pressed(egui::Key::Backspace)}) {
                self.remove_selected();
            };

            if response.hovered() {
                if response.clicked() {
                    if self.to_spawn.is_some() {
                        let gate = self.to_spawn.take();
                        if let Some(g) = gate {
                            if let Some(ptr) = response.interact_pointer_pos() {
                                // Adjust pointer position by the inverse of the zoom level
                                let adjusted_ptr_x = ptr.x / self.zoom;
                                let adjusted_ptr_y = ptr.y / self.zoom;
                            
                                // Round the adjusted pointer position to the nearest grid spacing
                                let x = (adjusted_ptr_x / GRID_SPACING).round() * GRID_SPACING;
                                let y = (adjusted_ptr_y / GRID_SPACING).round() * GRID_SPACING;
                            
                                // Adjust the pan offset by the inverse of the zoom level
                                let adjusted_pan_x = self.pan_offset.x / self.zoom;
                                let adjusted_pan_y = self.pan_offset.y / self.zoom;
                            
                                // Round the adjusted pan offset to the nearest grid spacing
                                let x_pan = (adjusted_pan_x / GRID_SPACING).round() * GRID_SPACING;
                                let y_pan = (adjusted_pan_y / GRID_SPACING).round() * GRID_SPACING;

                                // TODO
                                // Implement error handling when a property is read wronlgy
                                // that includes not reading at all because not present 
                                // and wrong len of inputs_pos in comparison to num_ins
                                // and wrong len of outputs_pos in comparison to num_outs
                                // auch checken ob die inputs und outputs pos in den richtigen grenzen liegen
                                // auch checken ob inputs und outputs gleihce einträge haben
                                let width_o = g.files.read_props().unwrap().width;
                                let height_o = g.files.read_props().unwrap().height;
                                let width;
                                let height;
                                if let Some(width_s) = width_o {
                                    width = width_s as f32 * GRID_SPACING;
                                }
                                else {
                                    width = 3.0 * GRID_SPACING;
                                }

                                if let Some(height_s) = height_o {
                                    height = height_s as f32 * GRID_SPACING;
                                }
                                else {
                                    height = 2.0 * GRID_SPACING;
                                }
                            
                                // Calculate the final position for the new gate, considering the adjusted and rounded values
                                self.add_gate(DrawableGate::from_ghost(ctx, g, (x - x_pan, y - y_pan), (width, height)));
                            }
                            
                        }
                    }

                    self.unselect_all();
                }

                // Handle panning
                self.pan_offset += response.drag_delta() / self.zoom;

                // Handle zooming
                let zoom_speed = 0.01;

                // Clamp the zoom level
                self.zoom *= (scroll_delta.y * zoom_speed).exp();
                self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
            }

            // Draw the grid
            self.draw_grid(&painter, response.rect);

            for gate in &mut self.gates {
                gate.draw(ctx, ui, &painter, self.pan_offset, self.zoom);
            }

            // Draw logic gates
            // for gate in &self.gates {
            //     self.draw_gate(&painter, gate);
            // }
        });
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        let grid_color = egui::Color32::from_gray(200); // Light gray for the grid lines
        let line_width = 0.2; // Width of the grid lines
    
        let grid_spacing = GRID_SPACING * self.zoom; // Adjust grid spacing based on the zoom level
    
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
}
