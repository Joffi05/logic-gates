use std::{cell::{Ref, RefCell}, rc::Rc};

use egui_sdl2_gl::egui::{self as egui, Color32, InputState, Response, Stroke};
use uuid::Uuid;
use crate::{ui::drawable_gate::DrawableGate, Circuit, LogicGate};

use super::{drawable_connection::DrawableConnection, drawable_gate::InOutPosition, event_queue::{CanvasEvent, EventQueue, GateEvent}, gate_list::GhostGate};

const MAX_ZOOM: f32 = 20.0;
const MIN_ZOOM: f32 = 0.3;
const ZOOM_SPEED: f32 = 0.01;
pub const GRID_SPACING: f32 = 20.0;

pub struct Canvas {
    pan_offset: egui::Vec2, // Current pan offset
    zoom: f32, // Current zoom level
    gates: Vec<Rc<RefCell<Box<DrawableGate>>>>, // List of gates on the canvas
    to_spawn: Option<GhostGate>,
    connections: Vec<DrawableConnection>,
    underlying_circuit: Circuit,
    selected_input: Option<(InOutPosition, Uuid)>,
    selected_output: Option<(InOutPosition, Uuid)>,
    events: EventQueue,
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
            selected_input: None,
            selected_output: None,
            events: EventQueue::new(),
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

    pub fn get_gate_by_id(&self, id: &Uuid) -> Option<Rc<RefCell<Box<DrawableGate>>>> {
        self.gates.iter().find(|gate| gate.borrow().id == *id).cloned()
    }
    
    pub fn add_gate(&mut self, gate: DrawableGate) {
        let gate_rc = Rc::new(RefCell::new(Box::new(gate)));
        let id = gate_rc.borrow().id.clone();

        let name = gate_rc.borrow().gate.borrow().get_name();
        if name == "BUTTON" {
            self.underlying_circuit.add_input(gate_rc.borrow().gate.clone());
        }
        else if name == "LAMP" {
            self.underlying_circuit.add_output(gate_rc.borrow().gate.clone());
        }
        else {
            self.underlying_circuit.add_gate(gate_rc.borrow().gate.clone(), id);
        }

        self.gates.push(gate_rc.clone());
    }

    pub fn add_connection(&mut self, connection: DrawableConnection) {
        // Now, connect the corresponding gates in the underlying circuit
        if let (Some(input_gate), Some(output_gate)) = (&connection.input_gate, &connection.output_gate) {
            // Assuming `DrawableConnection` holds the indexes for input/output
            // If not, you'll need to determine these based on your logic
            let input_index = connection.in_num.clone(); // Default to 0 or determine based on your logic
            let output_index = connection.out_num.clone(); // Default to 0 or determine based on your logic

            // Get indexes of the input/output gates
            let in_index = input_gate.borrow().outputs_pos.iter().position(|pos| pos.get() == output_index.get()).unwrap();
            let out_index = output_gate.borrow().inputs_pos.iter().position(|pos| pos.get() == input_index.get()).unwrap();

            // Call the connect method on the underlying circuit with the gates and their indexes
            self.underlying_circuit.connect(input_gate.borrow().gate.clone(), in_index, output_gate.borrow().gate.clone(), out_index);
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
    pub fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
            ctx.input(|i| {
                self.get_events(&response, i);
            });

            self.process_events(ctx);

            let zoom = self.get_zoom();
            let pan_offset = self.get_pan_offset();

            for conn in self.connections.iter_mut() {
                conn.update(zoom, pan_offset);
            }

            self.draw(ctx, ui,&painter, response.rect);
        
        });

        for g in &mut self.gates {
            g.borrow().gate.borrow_mut().calculate().unwrap();
        }

        self.underlying_circuit.calculate().unwrap();
    }

    fn get_events(&mut self, res: &Response, input: &InputState) {
        if input.key_pressed(egui::Key::Backspace) {
            self.events.add_event(CanvasEvent::RemoveSelected);
        }

        if res.hovered() {
            if input.scroll_delta.y != 0.0 {
                let new_zoom = (self.zoom + input.scroll_delta.y * ZOOM_SPEED).clamp(MIN_ZOOM, MAX_ZOOM);
                self.events.add_event(
                    CanvasEvent::ZoomCanvas {
                        from: self.zoom,
                        to: new_zoom,
                    }
                );
            }

            let mut dragged_gate = false;
            let mut clicked_in_out = false;

            // Assuming `is_gate_dragging` is a boolean field in your struct initialized to `false`
            if let Some(ptr) = res.interact_pointer_pos() {
                for g in self.gates.iter() {
                    if let Some(event) = g.borrow_mut().get_events(res, ptr, self.pan_offset, self.zoom) {
                        if let GateEvent::MovedGate { id, from, to, start } = &event {
                            dragged_gate = true; // Indicate that a gate is being dragged
                        }
                        if let GateEvent::ClickedIn { num, id } = &event {
                            clicked_in_out = true;
                        }
                        if let GateEvent::ClickedOut { num, id } = &event {
                            clicked_in_out = true;
                        }

                        self.events.add_event(CanvasEvent::GateEvent(event));
                    }
                }
            }

            let mut doubley = false;
            if res.double_clicked() {
                doubley = true;
                if let Some(ptr) = res.interact_pointer_pos() {
                    self.events.add_event(CanvasEvent::DoubleClickedCanvas { pos: ptr.into() });
                }
            }
        
            if res.clicked() && !doubley {
                if let Some(gate) = self.to_spawn.take() {
                    if let Some(ptr) = res.interact_pointer_pos() {
                        // TODO
                        // den laal hier verbessern
                        let adjusted_ptr_x = ptr.x / self.zoom;
                        let adjusted_ptr_y = ptr.y / self.zoom;
                    
                        let x = (adjusted_ptr_x / GRID_SPACING).round() * GRID_SPACING;
                        let y = (adjusted_ptr_y / GRID_SPACING).round() * GRID_SPACING;
                    
                        let adjusted_pan_x = self.pan_offset.x / self.zoom;
                        let adjusted_pan_y = self.pan_offset.y / self.zoom;
                    
                        let x_pan = (adjusted_pan_x / GRID_SPACING).round() * GRID_SPACING;
                        let y_pan = (adjusted_pan_y / GRID_SPACING).round() * GRID_SPACING;
                    
                        // TODO
                        // Implement error handling when a property is read wronlgy
                        // that includes not reading at all because not present 
                        // and wrong len of inputs_pos in comparison to num_ins
                        // and wrong len of outputs_pos in comparison to num_outs
                        // auch checken ob die inputs und outputs pos in den richtigen grenzen liegen
                        // auch checken ob inputs und outputs gleihce einträge haben
                        let width_o = gate.files.read_props().unwrap().width;
                        let height_o = gate.files.read_props().unwrap().height;
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
                    
                        self.events.add_event(
                            CanvasEvent::SpawnGate {
                                gate: gate.to_owned(),
                                pos: (x - x_pan, y - y_pan),
                                size: (width, height),
                            }
                        );
                    }
                }
                else {
                    if let Some(ptr) = res.interact_pointer_pos() {
                        if !clicked_in_out {
                            self.events.add_event(CanvasEvent::ClickedCanvas { pos: (ptr.x, ptr.y) });
                        }
                    }
                }
            }

            if res.dragged() {
                if !dragged_gate && (res.drag_delta().x.abs() > 0.01 || res.drag_delta().y.abs() > 0.01) { // Only pan the canvas if no gate is being dragged
                    self.events.add_event(
                        CanvasEvent::PanCanvas {
                            from: (self.pan_offset.x, self.pan_offset.y),
                            to: (self.pan_offset.x + res.drag_delta().x, self.pan_offset.y + res.drag_delta().y),
                        }
                    );
                }
            }
        }
    }

    fn process_events(&mut self, ctx: &egui::Context) {
        let mut event_to_add: Option<CanvasEvent> = None;

        let current_event = self.events.get_current();

        if let Some(event) = current_event {
            match event {
                CanvasEvent::SpawnGate { gate, pos, size } => {
                    self.add_gate(DrawableGate::from_ghost(ctx, gate.clone(), *pos, *size));
                }
                CanvasEvent::AddConnection { from_gate, to_gate, InputPos, OutputPos } => {
                    let connection = DrawableConnection::with_gates(
                        from_gate.borrow().get_pos_of_in_out(OutputPos.clone(), self.zoom, self.pan_offset),
                        to_gate.borrow().get_pos_of_in_out(InputPos.clone(), self.zoom, self.pan_offset),
                        InputPos.clone(),
                        OutputPos.clone(),
                        Color32::WHITE,
                        from_gate.clone(),
                        to_gate.clone(),
                        Uuid::new_v4()
                    );

                    self.add_connection(connection);
                }
                CanvasEvent::SplitterClicked { pos, gate } => {
                    if let Some(sel_inp) = &self.selected_input {
                        let connection = DrawableConnection::with_gates(
                            gate.borrow().get_pos_of_in_out(sel_inp.0.clone(), self.zoom, self.pan_offset),
                            (pos.0, pos.1),
                            sel_inp.0.clone(),
                            InOutPosition::new(0),
                            Color32::WHITE,
                            gate.clone(),
                            gate.clone(),
                            Uuid::new_v4()
                        );

                        self.add_connection(connection);
                    }
                    else if let Some(sel_out) = &self.selected_output {
                        let connection = DrawableConnection::with_gates(
                            (pos.0, pos.1),
                            gate.borrow().get_pos_of_in_out(sel_out.0.clone(), self.zoom, self.pan_offset),
                            InOutPosition::new(0),
                            sel_out.0.clone(),
                            Color32::WHITE,
                            gate.clone(),
                            gate.clone(),
                            Uuid::new_v4()
                        );

                        self.add_connection(connection);
                    }
                }
                CanvasEvent::RemoveSelected => {
                    self.remove_selected();
                }
                CanvasEvent::PanCanvas { from, to } => {
                    self.pan_offset = egui::Vec2::new(to.0, to.1);
                }
                CanvasEvent::ZoomCanvas { from, to } => {
                    self.zoom = *to;
                }
                CanvasEvent::ClickedCanvas { pos } => {
                    self.unselect_all();
                    self.selected_input = None;
                    self.selected_output = None;
                }
                CanvasEvent::DoubleClickedCanvas { pos } => {
                    
                }
                CanvasEvent::RightClickedCanvas { pos } => {
                    // TODO
                    // Implement context menu
                }
                CanvasEvent::GateEvent(event) => {
                    for g in self.gates.iter() {
                        match event {
                            GateEvent::ClickedOn { id } => {
                                let mut g_ref = g.borrow_mut();
                                if g_ref.id == *id {
                                    g_ref.selected = true;
                        
                                    let is_button = g_ref.gate.borrow().get_name() == "BUTTON";
                                    
                                    drop(g_ref); 
                        
                                    if is_button {
                                        let gate_ref = g.borrow_mut(); 
                                        let mut gate_logic_ref = gate_ref.gate.borrow_mut(); 
                        
                                        if let Some(memory) = gate_logic_ref.get_memory() {
                                            gate_logic_ref.set_memory(0, !memory[0]);
                                        }
                                    }
                                }  
                            },
                            GateEvent::ClickedIn { num, id } => {
                                if g.borrow().id == *id {
                                    if let Some((num_out, id_out)) = &self.selected_output.take() {
                                        event_to_add = Some(CanvasEvent::AddConnection { from_gate: self.get_gate_by_id(id_out).unwrap(), to_gate: g.clone(), InputPos: num.clone(), OutputPos: num_out.clone() });
                                    }
                                    else {
                                        self.selected_input = Some((num.clone(), id.clone()));
                                    }
                                    
                                    if g.borrow().gate.borrow().get_name() == "SPLITTER" {
                                        event_to_add = Some(CanvasEvent::SplitterClicked { pos: g.borrow().pos, gate: g.clone() });
                                    }
                                }
                            },
                            GateEvent::ClickedOut { num, id } => {
                                if g.borrow().id == *id {
                                    if let Some((num_in, id_in)) = &self.selected_input.take() {
                                        event_to_add = Some(CanvasEvent::AddConnection { from_gate: g.clone(), to_gate: self.get_gate_by_id(id_in).unwrap(), InputPos: num_in.clone(), OutputPos: num.clone() });
                                    }
                                    else {
                                        self.selected_output = Some((num.clone(), id.clone()));
                                    }
                                }
                            },
                            GateEvent::MovedGate { id, from, to, start } => {
                                if g.borrow().id == *id {
                                    // Create a single mutable borrow at the start
                                    let mut g_ref = g.borrow_mut();
                                    
                                    g_ref.drag.0 += to.0 - from.0;
                                    g_ref.drag.1 += to.1 - from.1;
                                
                                    // Directly use the mutable borrow without additional calls to `borrow_mut`
                                    // Check if the drag distance exceeds the grid spacing in either direction
                                    if g_ref.drag.0.abs() >= GRID_SPACING || g_ref.drag.1.abs() >= GRID_SPACING {
                                        // Snap the position to the nearest grid line by adding the accumulated drag distance
                                        // and then rounding to the nearest GRID_SPACING multiple
                                        g_ref.pos.0 = ((g_ref.pos.0 + g_ref.drag.0) / GRID_SPACING).round() * GRID_SPACING;
                                        g_ref.pos.1 = ((g_ref.pos.1 + g_ref.drag.1) / GRID_SPACING).round() * GRID_SPACING;
                                
                                        // Reset the drag accumulator since we've just snapped the gate to the grid
                                        g_ref.drag.0 = 0.0;
                                        g_ref.drag.1 = 0.0;
                                    }
                                }
                            },
                        }
                    }
                }
                _ => {}
            }

            self.events.advance();
        }

        // After processing the current event, check if there's a new event to add
        if let Some(new_event) = event_to_add {
            // Add the new event to the queue and check if a new event was actually added
            let new_event_added = self.events.add_event(new_event);
        }
    }

    fn draw(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
        self.draw_grid(painter, rect);
        for gate in &self.gates {
            gate.borrow_mut().draw(ctx, ui, painter, self.pan_offset, self.zoom);

            if let Some(sel_in) = &self.selected_input {
                if sel_in.1 == gate.borrow().id {
                    // Draw a white circle around the selected input to signal that its selected
                    let (x, y) = gate.borrow().get_pos_of_in_out(sel_in.0.clone(), self.zoom, self.pan_offset);
                    let pos = egui::pos2(x, y);
                    let radius = 10.0;
                    let color = Color32::WHITE;
                    painter.circle_stroke(pos, radius, Stroke::new(2.0, color));
                }
            }

            if let Some(sel_out) = &self.selected_output {
                if sel_out.1 == gate.borrow().id {
                    // Draw a white circle around the selected output to signal that its selected
                    let (x, y) = gate.borrow().get_pos_of_in_out(sel_out.0.clone(), self.zoom, self.pan_offset);
                    let pos = egui::pos2(x, y);
                    let radius = 10.0;
                    let color = Color32::WHITE;
                    painter.circle_stroke(pos, radius, Stroke::new(2.0, color));
                }
            }
        }
        for connection in &self.connections {
            connection.draw(painter, self.pan_offset, self.zoom);
        }
    }

    // // Method to render the canvas and its contents
    // pub fn draw_old(&mut self, ctx: &egui::Context) {
    //     egui::CentralPanel::default().show(ctx, |ui| {
    //         let mut clicked_in: Option<((f32, f32), Rc<RefCell<Box<DrawableGate>>>, InOutPosition)> = None;
    //         let mut clicked_out: Option<((f32, f32), Rc<RefCell<Box<DrawableGate>>>, InOutPosition)> = None;
    //         let mut gates_to_reset = Vec::new();
    //         let mut just_connected: bool = false;
    //         // Collect connection information
    //         for gate in &self.gates {
    //             let interacted = gate.borrow_mut().interaction_logic(ctx, ui, &painter, self.pan_offset, self.zoom);
                
    //             gate.borrow_mut().draw(ctx, ui, &painter, self.pan_offset, self.zoom);

    //             let gate_ref = gate.borrow();
    //             if let Some((pos, index, id)) = &gate_ref.in_clicked_at {
    //                 clicked_in = Some(((pos.0, pos.1), gate.clone(), index.clone()));
    //                 gates_to_reset.push(gate.clone());
    //             }
    //             if let Some((pos, index, id)) = &gate_ref.out_clicked_at {
    //                 clicked_out = Some(((pos.0, pos.1), gate.clone(), index.clone()));
    //                 gates_to_reset.push(gate.clone());
    //             }

    //             if interacted {
    //                 let zoom_level = self.zoom;
    //                 for conn in self.connections.iter_mut() {
    //                     if let (Some(inp), Some(out)) = (conn.input_gate.clone(), conn.output_gate.clone()) {
    //                         if Rc::ptr_eq(&inp, gate) {
    //                             // The current gate is the input gate for this connection
    //                             // Update the start point of the connection
    //                             let input_gate = inp.borrow();
    //                             let (x, y)  = input_gate.get_pos_of_in_out(conn.in_num.clone(), zoom_level, self.pan_offset);
                                
    //                             let canvas_x = (x - self.pan_offset.x) / zoom_level;
    //                             let canvas_y = (y - self.pan_offset.y) / zoom_level;
                                
    //                             conn.start = (canvas_x, canvas_y);
    //                         }
                
    //                         if Rc::ptr_eq(&out, gate) {
    //                             // The current gate is the output gate for this connection
    //                             // Update the end point of the connection
    //                             let output_gate = out.borrow();
    //                             let (x, y)  = output_gate.get_pos_of_in_out(conn.out_num.clone(), zoom_level, self.pan_offset);
                                
    //                             let canvas_x = (x - self.pan_offset.x) / zoom_level;
    //                             let canvas_y = (y - self.pan_offset.y) / zoom_level;
                                
    //                             conn.end = (canvas_x, canvas_y);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
            
    //         // Handle connections based on collected information
    //         if let (Some((in_pos, in_gate, in_index)), Some((out_pos, out_gate, out_index))) = (clicked_in, clicked_out) {
    //             let connection = DrawableConnection::with_gates(
    //                 in_pos,  // Store raw position
    //                 out_pos,  // Store raw position
    //                 in_index,
    //                 out_index,
    //                 Color32::WHITE, 
    //                 in_gate.clone(), 
    //                 out_gate.clone(), 
    //                 Uuid::new_v4()
    //             );                
    //             self.add_connection(connection);
    //             just_connected = true;
    //         }

    //         // Clear the clicked_at fields only for gates involved in a new connection
    //         if just_connected {
    //             for gate in gates_to_reset {
    //                 let mut gate_ref = gate.borrow_mut();
    //                 gate_ref.in_clicked_at = None;
    //                 gate_ref.out_clicked_at = None;
    //             }
    //             just_connected = false;
    //         }

    //         // Draw the connections
    //         for connection in &self.connections {
    //             connection.draw(&painter, self.pan_offset, self.zoom);
    //         }
    //     });
    // }

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
