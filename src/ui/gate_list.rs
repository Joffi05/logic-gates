
use egui_sdl2_gl::egui::{self as egui, pos2};

use crate::BasicGate;
use crate::LogicGate;


use super::drawable_gate::GateFiles;
use super::drawable_gate::InOutPosition;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

pub struct GhostGate {
    pub gate: Rc<RefCell<Box<dyn LogicGate>>>,
    pub files: GateFiles,
    pub inputs_pos: Vec<InOutPosition>,
    pub outputs_pos: Vec<InOutPosition>,
}

impl PartialEq for GhostGate {
    fn eq(&self, other: &Self) -> bool {
        self.gate.borrow().get_name() == other.gate.borrow().get_name()
    }
}

impl Clone for GhostGate {
    fn clone(&self) -> Self {
        Self {
            gate: Rc::new(RefCell::new(Box::new(BasicGate::from_lua(self.gate.borrow().get_name(), self.files.lua.clone()).unwrap()))),
            files: self.files.clone(),
            inputs_pos: self.inputs_pos.clone(),
            outputs_pos: self.outputs_pos.clone(),
        }
    }
}


pub struct GateList {
    buttons: Vec<GhostGate>,
    pinned: bool,
    open: bool,
    anchor: [f32; 2],
    pub gate_to_spawn: Option<GhostGate>,
}

impl GateList {
    pub fn new() -> Self {

        Self { buttons: vec![], pinned: false, open: true, anchor: [0.0, 0.0], gate_to_spawn: None}
    }

    fn add_gate(&mut self, gate: GhostGate) {
        self.buttons.push(gate);
    }

    pub fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub fn get_open(&self) -> bool {
        self.open
    }

    pub fn update(&mut self, _ctx: &egui::Context) {
        // Read the gates from ./comps
        let comps_dir = "./comps";
        if let Ok(entries) = fs::read_dir(comps_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".lua") {
                            let json_file_name = file_name.replace(".lua", ".json");
                            let json_file = Path::new(comps_dir).join(json_file_name);
                            
                            let gate_file;
                            if json_file.exists() {
                                gate_file = GateFiles {
                                    lua: entry.path().into_boxed_path(),
                                    json: Some(json_file.into_boxed_path()),
                                };
                            } else {
                                gate_file = GateFiles {
                                    lua: entry.path().into_boxed_path(),
                                    json: None,
                                };
                            }

                            // Hier auch prop reading abchecken
                            // props müssen zentral gespeichert werden.
                            let props = gate_file.read_props().unwrap();
                            let ins = props.inputs_pos;
                            let outs = props.outputs_pos;

                            let gate_name = file_name.split(".").next().unwrap().to_ascii_uppercase();

                            let gate = GhostGate {
                                    gate: Rc::new(RefCell::new(Box::new(BasicGate::from_lua(gate_name, gate_file.lua.clone()).unwrap()))),
                                    files: gate_file,
                                    inputs_pos: ins,
                                    outputs_pos: outs,
                            };

                            // Add if not already in
                            if !self.buttons.contains(&gate) {
                                self.add_gate(gate);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        
        let mut window = egui::Window::new("Gates").resizable(true);
        if self.pinned {
            window = window.fixed_pos(pos2(self.anchor[0], self.anchor[1]));
            window = window.resizable(false);
        }

        // Open the window
        let response = window
            .title_bar(false)
            .show(ctx, |ui| {
                // Custom title bar
                ui.horizontal(|ui| {
                    ui.label("Gates"); // Custom title
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close button
                        if ui.button("x").clicked() {
                            self.open = false;
                        }
                        if ui.button("pin").clicked() {
                            self.pinned = !self.pinned;
                        }
                    });
                });

                if self.pinned {
                    
                }

                // Directly modify the style of the ui
                let style = ui.style_mut();
    
                style.visuals.button_frame = true;
                style.visuals.menu_rounding = egui::Rounding::ZERO;
                style.visuals.window_rounding = egui::Rounding::ZERO;

                ctx.set_style(style.clone());
    
                // The ScrollArea takes up the rest of the space
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for gate in &self.buttons {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                            // Add a button for each label
                            let bt_res = ui.button(gate.gate.borrow().get_name());
                            if bt_res.clicked() {
                                self.gate_to_spawn = Some(gate.clone());
                            }
                            if bt_res.double_clicked() {
                                println!("Double clicked: {}", gate.gate.borrow().get_name());
                            }
                        });
                    }
                });

                ui.allocate_space(ui.available_size());
            });

        if let Some(response) = response {
            self.anchor = response.response.rect.left_top().into();
        }
    }
    
    
}