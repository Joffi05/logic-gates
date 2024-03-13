
use egui_sdl2_gl::egui::{self as egui, pos2};

pub struct ButtonList {
    buttons: Vec<String>, // List of button labels
    pinned: bool,
    open: bool,
    anchor: [f32; 2],
}

impl ButtonList {
    pub fn new() -> Self {
        Self { buttons: vec![], pinned: false, open: true, anchor: [0.0, 0.0]}
    }

    pub fn add_button(&mut self, label: &str) {
        self.buttons.push(label.to_owned());
    }

    pub fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub fn get_open(&self) -> bool {
        self.open
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

                // Position the "Add button" at the top, center-aligned
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                    if ui.button("Add button").clicked() {
                        let count = self.buttons.len() + 1;
                        self.add_button(&format!("Button {}", count));
                    }
                });
    
                // The ScrollArea takes up the rest of the space
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for label in &self.buttons {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                            // Add a button for each label
                            if ui.button(label).clicked() {
                                println!("Clicked: {}", label);
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