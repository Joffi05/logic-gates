use egui_sdl2_gl::egui as egui;
use crate::ui::button_list;
use crate::ui::top_menu;
use crate::ui::canvas;

pub struct State {
    pub canvas: canvas::Canvas,
    pub top_menu: top_menu::TopMenu,
    pub gate_selector: Option<button_list::ButtonList>,
}

impl State {
    pub fn new() -> Self {
        Self {
            canvas: canvas::Canvas::new(),
            top_menu: top_menu::TopMenu::new(),
            gate_selector: None,
        }
    }
    fn update(&mut self, ctx: &egui::Context) {
        //Fucking ui programming stateful shitt ffuck ass aids 
        // FIX unfassbar schlecht
        if self.top_menu.open_gate_selector {
            if let Some(sel) = &mut self.gate_selector {
                if !sel.get_open() {
                    sel.set_open(true);
                }
            }
            else {
                self.gate_selector = Some(button_list::ButtonList::new());
            }
                
            self.top_menu.open_gate_selector = false;
        }

        if self.top_menu.jump_to_0_0 {
            self.canvas.jump_to(0.0, 0.0);
        }
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        self.update(ctx);

        self.canvas.draw(ctx);

        self.top_menu.show(ctx);
        if let Some(gate_selector) = &mut self.gate_selector {
            gate_selector.show(ctx);
        }
    }
}