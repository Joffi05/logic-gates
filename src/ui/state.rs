

use egui_sdl2_gl::egui as egui;
use crate::ui::gate_list;
use crate::ui::top_menu;

use super::canvas_list::CanvasList;

use super::gate_list::GateList;

pub struct State {
    pub canvas_list: CanvasList,
    pub top_menu: top_menu::TopMenu,
    pub gate_selector: Option<gate_list::GateList>,
}

impl State {
    pub fn new() -> Self {
        let mut state = Self {
            canvas_list: CanvasList::new(),
            top_menu: top_menu::TopMenu::new(),
            gate_selector: Some(GateList::new()),
        };
        state.top_menu.open_gate_selector = true;
        state
    }

    fn update(&mut self, ctx: &egui::Context) {
        //Fucking ui programming stateful shitt ffuck ass aids 
        // FIX unfassbar schlecht
        if self.top_menu.open_gate_selector {
            if let Some(sel) = &mut self.gate_selector {                
                if !sel.get_open() {
                    sel.set_open(true);
                }

                sel.update(ctx);
            }
            else {
                self.gate_selector = Some(gate_list::GateList::new());
                self.gate_selector.as_mut().unwrap().update(ctx);
            }
            
            self.top_menu.open_gate_selector = false;
        }

        if let Some(sel) = &mut self.gate_selector {
            if let Some(_) = &mut sel.gate_to_spawn {
                if let Some(canvas) = self.canvas_list.get_selected() {
                    let gate = sel.gate_to_spawn.take();
                    if let Some(g) = gate {
                        canvas.add_to_spawn(g);
                    }
                }
            }
        }

        if self.top_menu.jump_to_0_0 {
            if let Some(canvas) = self.canvas_list.get_selected() {
                canvas.jump_to(0.0, 0.0);
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.update(ctx);

        if let Some(canvas) = self.canvas_list.get_selected() {
            canvas.draw(ctx);
        }

        self.top_menu.show(ctx);

        egui::TopBottomPanel::top("canvas_list").show(ctx, |ui| {
            self.canvas_list.show(ui);
        });

        // Panel for displaying the mouse location
        // Display the mouse location as a floating label
        egui::Area::new("mouse_location")
            .fixed_pos(ctx.available_rect().right_top() - egui::vec2(120.0, 0.0)) // Adjust these values to position the label
            .show(ctx, |ui| {
                if let Some(pointer_pos) = ctx.input(|i| i.pointer.hover_pos()) {
                    let (pan_x_dif, pan_y_dif) = self.canvas_list.get_selected().map_or((0.0, 0.0), |canvas| {
                        let pan_offset = canvas.get_pan_offset();
                        (pan_offset.x, pan_offset.y)
                    });

                    let text_color = egui::Color32::WHITE; // Example: Red color
                    ui.visuals_mut().widgets.noninteractive.fg_stroke.color = text_color;

                    // Display the mouse position with adjustments for pan offsets and the zoom level
                    if let Some(zoom) = self.canvas_list.get_selected() {
                        ui.label(format!("({:.1}, {:.1}) : {:.1}", pointer_pos.x - pan_x_dif, pointer_pos.y - pan_y_dif, zoom.get_zoom()));
                    }
                    else {
                        ui.label(format!("({:.1}, {:.1})", pointer_pos.x - pan_x_dif, pointer_pos.y - pan_y_dif));
                    }
                }
            });


        if let Some(gate_selector) = &mut self.gate_selector {
            gate_selector.show(ctx);
        }
    }
}