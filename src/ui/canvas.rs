use std::{cell::{Ref, RefCell}, rc::Rc};

use egui_sdl2_gl::egui::{self as egui, Color32, Rect};
use uuid::Uuid;
use crate::{ui::drawable_gate::DrawableGate, Circuit};

use super::{drawable_connection::DrawableConnection, drawable_gate::InOutPosition, gate_list::GhostGate};

const MAX_ZOOM: f32 = 20.0;
const MIN_ZOOM: f32 = 0.3;
pub const GRID_SPACING: f32 = 20.0;

pub struct Canvas {
    pan_offset: egui::Vec2, // Current pan offset
    zoom: f32, // Current zoom level
    gates: Vec<Rc<RefCell<Box<DrawableGate>>>>, // List of gates on the canvas
    to_spawn: Option<GhostGate>,
    connections: Vec<DrawableConnection>,
    underlying_circuit: Circuit,
}

impl Canvas {
    pub fn new(name: &str) -> Self {
        Canvas {
            pan_offset: egui::Vec2::ZERO,
            zoom: 1.0,
            gates: vec![],
            to_spawn: None,
            connections: vec![],
            underlying_circuit: Circuit::new(name.to_string()),
        }
    }

    pub fn get_pan_offset(&self) -> egui::Vec2 {
        self.pan_offset
    }

    pub fn get_conn_len(&self) -> usize {
        self.connections.len()
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn add_gate(&mut self, gate: DrawableGate) {
        let gate_rc = Rc::new(RefCell::new(Box::new(gate)));
        let id = gate_rc.borrow().id.clone();
        self.underlying_circuit.add_gate(gate_rc.borrow().gate.clone(), id);
        self.gates.push(gate_rc.clone());
    }

    pub fn add_connection(&mut self, connection: DrawableConnection) {
        // Now, connect the corresponding gates in the underlying circuit
        if let (Some(input_gate), Some(output_gate)) = (&connection.input_gate, &connection.output_gate) {
            // Assuming `DrawableConnection` holds the indexes for input/output
            // If not, you'll need to determine these based on your logic
            let input_index = connection.in_num.clone(); // Default to 0 or determine based on your logic
            let output_index = connection.out_num.clone(); // Default to 0 or determine based on your logic
    
            // Call the connect method on the underlying circuit with the gates and their indexes
            self.underlying_circuit.connect(input_gate.borrow().gate.clone(), input_index.get() as usize, output_gate.borrow().gate.clone(), output_index.get() as usize);
            println!("Underlying circuit: {:?}", self.get_conn_len());
        }

        // Add the DrawableConnection to the list of connections
        self.connections.push(connection);
    }
    

    pub fn remove_selected(&mut self) {
        // First, collect Rc pointers to the selected gates.
        let removed_gates: Vec<Rc<RefCell<Box<DrawableGate>>>> = self.gates.iter()
            .filter(|gate| gate.borrow().selected)
            .cloned()  // Clone the Rc pointers, not the gates themselves.
            .collect(); 
    
        // Remove the selected gates.
        self.gates.retain(|gate| !gate.borrow().selected);
    
        // Remove connections associated with the removed gates.
        self.connections.retain(|connection| {
            // Check if the input_gate or output_gate of the connection is among the removed gates.
            let input_gate_linked = connection.input_gate.as_ref()
                .map_or(false, |input_gate| removed_gates.iter().any(|rg| Rc::ptr_eq(rg, input_gate)));
            let output_gate_linked = connection.output_gate.as_ref()
                .map_or(false, |output_gate| removed_gates.iter().any(|rg| Rc::ptr_eq(rg, output_gate)));
    
            // Retain the connection only if neither its input_gate nor output_gate was removed.
            !input_gate_linked && !output_gate_linked
        });
    }
    

    pub fn unselect_all(&mut self) {
        for gate_rc in &self.gates {
            gate_rc.borrow_mut().selected = false;
        }
    }

    pub fn add_to_spawn(&mut self, gate: GhostGate) {
        self.to_spawn = Some(gate);
    }

    pub fn jump_to(&mut self, x: f32, y: f32) {
        self.pan_offset = egui::Vec2::new(x, y);
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

                // Handle panninga
                self.pan_offset += response.drag_delta();

                // Handle zooming
                let zoom_speed = 0.01;

                // Clamp the zoom level
                self.zoom *= (scroll_delta.y * zoom_speed).exp();
                self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
            }

            // Draw the grid
            self.draw_grid(&painter, response.rect);

            // First, handle drawing and collect information for connections if needed.
            let mut clicked_in: Option<((f32, f32), Rc<RefCell<Box<DrawableGate>>>, InOutPosition)> = None;
            let mut clicked_out: Option<((f32, f32), Rc<RefCell<Box<DrawableGate>>>, InOutPosition)> = None;
            let mut gates_to_reset = Vec::new();
            let mut just_connected: bool = false;
            // Collect connection information
            for gate in &self.gates {
                let interacted = gate.borrow_mut().interaction_logic(ctx, ui, &painter, self.pan_offset, self.zoom);
                
                gate.borrow_mut().draw(ctx, ui, &painter, self.pan_offset, self.zoom);

                let gate_ref = gate.borrow();
                if let Some((pos, index, id)) = &gate_ref.in_clicked_at {
                    println!("In clicked at: {:?}", pos.0);
                    clicked_in = Some(((pos.0, pos.1), gate.clone(), index.clone()));
                    gates_to_reset.push(gate.clone());
                }
                if let Some((pos, index, id)) = &gate_ref.out_clicked_at {
                    println!("Out clicked at: {:?}", pos.0);
                    clicked_out = Some(((pos.0, pos.1), gate.clone(), index.clone()));
                    gates_to_reset.push(gate.clone());
                }

                if interacted {
                    let zoom_level = self.zoom;
                    for conn in self.connections.iter_mut() {
                        if let (Some(inp), Some(out)) = (conn.input_gate.clone(), conn.output_gate.clone()) {
                            if Rc::ptr_eq(&inp, gate) {
                                // The current gate is the input gate for this connection
                                // Update the start point of the connection
                                let input_gate = inp.borrow();
                                let (x, y)  = input_gate.get_pos_of_in_out(conn.in_num.clone(), gate.borrow().get_rect(zoom_level, self.pan_offset), zoom_level);
                                
                                let canvas_x = (x - self.pan_offset.x) / zoom_level;
                                let canvas_y = (y - self.pan_offset.y) / zoom_level;
                                
                                conn.start = (canvas_x, canvas_y);
                            }
                
                            if Rc::ptr_eq(&out, gate) {
                                // The current gate is the output gate for this connection
                                // Update the end point of the connection
                                let output_gate = out.borrow();
                                let (x, y)  = output_gate.get_pos_of_in_out(conn.out_num.clone(), gate.borrow().get_rect(zoom_level, self.pan_offset), zoom_level);
                                
                                let canvas_x = (x - self.pan_offset.x) / zoom_level;
                                let canvas_y = (y - self.pan_offset.y) / zoom_level;
                                
                                conn.end = (canvas_x, canvas_y);
                            }
                        }
                    }
                }
            }
            
            // Handle connections based on collected information
            if let (Some((in_pos, in_gate, in_index)), Some((out_pos, out_gate, out_index))) = (clicked_in, clicked_out) {
                let connection = DrawableConnection::with_gates(
                    in_pos,  // Store raw position
                    out_pos,  // Store raw position
                    in_index,
                    out_index,
                    Color32::WHITE, 
                    in_gate.clone(), 
                    out_gate.clone(), 
                    Uuid::new_v4()
                );                
                self.add_connection(connection);
                just_connected = true;
            }

            // Clear the clicked_at fields only for gates involved in a new connection
            if just_connected {
                for gate in gates_to_reset {
                    let mut gate_ref = gate.borrow_mut();
                    gate_ref.in_clicked_at = None;
                    gate_ref.out_clicked_at = None;
                }
                just_connected = false;
            }

            // Draw the connections
            for connection in &self.connections {
                connection.draw(&painter, self.pan_offset, self.zoom);
            }
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
