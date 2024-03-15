use std::rc::Rc;

use egui_sdl2_gl::egui::{self as egui, Color32};

use super::drawable_gate::DrawableGate;

pub struct DrawableConnections {
    pub start: (f32, f32), // (x, y) of the start of the connection (in canvas space)
    pub end: (f32, f32), // (x, y) of the end of the connection (in canvas space)
    pub color: Color32,
    pub input_gate: Rc<DrawableGate>,
    pub output_gate: Rc<DrawableGate>,
}