use std::{cell::RefCell};
use std::rc::Rc;
use egui_sdl2_gl::egui::{self as egui, Color32};

use super::drawable_gate::{DrawableGate, InOutPosition};

pub struct DrawableConnection {
    pub start: (f32, f32), // (x, y) of the start of the connection (in canvas space)
    pub end: (f32, f32), // (x, y) of the end of the connection (in canvas space)
    pub in_num: InOutPosition,
    pub out_num: InOutPosition,
    pub color: Color32,
    pub input_gate: Option<Rc<RefCell<Box<DrawableGate>>>>,
    pub output_gate: Option<Rc<RefCell<Box<DrawableGate>>>>,
    pub id: uuid::Uuid,
}

impl DrawableConnection {
    pub fn new(start: (f32, f32), end: (f32, f32), in_num: InOutPosition, out_num: InOutPosition, color: Color32, id: uuid::Uuid) -> Self {
        DrawableConnection {
            start,
            end,
            in_num,
            out_num,
            color,
            input_gate: None,
            output_gate: None,
            id,
        }
    }

    pub fn with_gates(
        start: (f32, f32),
        end: (f32, f32),
        in_num: InOutPosition,
        out_num: InOutPosition,
        color: Color32,
        input_gate: Rc<RefCell<Box<DrawableGate>>>,
        output_gate: Rc<RefCell<Box<DrawableGate>>>,
        id: uuid::Uuid,
    ) -> Self {
        DrawableConnection {
            start,
            end,
            in_num,
            out_num,
            color,
            input_gate: Some(input_gate),
            output_gate: Some(output_gate),
            id,
        }
    }

    pub fn update(&mut self, zoom_level: f32, pan_offset: egui::Vec2) {
        // Update the start point if there's an input gate
        if let Some(input_gate) = &self.input_gate {
            let (in_x, in_y) = input_gate.borrow().get_pos_of_in_out(self.out_num.clone(), zoom_level, pan_offset);
    
            // Adjust the position from world space to canvas space
            self.start.0 = in_x - pan_offset.x;
            self.start.1 = in_y - pan_offset.y;
        }
    
        // Update the end point if there's an output gate
        if let Some(output_gate) = &self.output_gate {
            let (out_x, out_y) = output_gate.borrow().get_pos_of_in_out(self.in_num.clone(), zoom_level, pan_offset);
    
            // Adjust the position from world space to canvas space
            self.end.0 = out_x - pan_offset.x;
            self.end.1 = out_y - pan_offset.y;
        }
    }    

    pub fn draw(&self, painter: &egui::Painter, pan_offset: egui::Vec2, zoom_level: f32) {
        // Apply current view transformations to the canvas space coordinates
        let start_adjusted = egui::pos2(
            self.start.0 + pan_offset.x,
            self.start.1 + pan_offset.y
        );
        let end_adjusted = egui::pos2(
            self.end.0 + pan_offset.x,
            self.end.1 + pan_offset.y
        );
    
        // Draw the line with adjusted coordinates
        painter.line_segment([start_adjusted, end_adjusted], (1.0 * zoom_level, self.color));
    }
}