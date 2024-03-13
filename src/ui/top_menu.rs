use egui_sdl2_gl::egui as egui;

pub struct TopMenu {
    pub open_gate_selector: bool,
    pub jump_to_0_0: bool,
}

impl TopMenu {
    pub fn new() -> Self {
        Self {
            open_gate_selector: false,
            jump_to_0_0: false,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        // Handle the New action
                        println!("New file");
                        ui.close_menu();
                        todo!("Handle the New action")
                    }
                    if ui.button("Open").clicked() {
                        // Handle the Open action
                        println!("Open file");
                        ui.close_menu();
                        todo!("Handle the Open action")
                    }
                    if ui.button("Save").clicked() {
                        // Handle the Save action
                        println!("Save file");
                        ui.close_menu();
                        todo!("Handle the Save action")
                    }
                    if ui.button("Quit").clicked() {
                        // Handle the Quit action
                        println!("Quit application");
                        // Logic to quit the application goes here
                        ui.close_menu();
    
                        // This is a hack to quit the application 
                        // TODO - Find a better way to quit the application
                        std::process::exit(0);
                    }
                });
    
                ui.menu_button("Edit", |ui| {
                    if ui.button("Undo").clicked() {
                        // Handle the Undo action
                        println!("Undo action");
                        ui.close_menu();
                        todo!("Handle the Undo action")
                    }
                    if ui.button("Redo").clicked() {
                        // Handle the Redo action
                        println!("Redo action");
                        ui.close_menu();
                        todo!("Handle the Redo action")
                    }
                    if ui.button("Jump to 0:0").clicked() {
                        // Handle the Jump to 0:0 action
                        println!("Jump to 0:0 action");
                        self.jump_to_0_0 = true;
                        ui.close_menu();
                    }
                    // Add more Edit actions here
                });
    
                ui.menu_button("Windows", |ui| {
                    if ui.button("Gate Selector").clicked() {
                        // Handle the Gate Selector action
                        println!("Gate Selector");
                        self.open_gate_selector = true;
                        ui.close_menu();
                    }
                })
    
                // Add more top-level menus as needed
            });
        });
    }
}