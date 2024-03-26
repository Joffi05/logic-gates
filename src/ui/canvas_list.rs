use egui_sdl2_gl::egui as egui;

use crate::ui::canvas::Canvas;

pub struct SelectableCanvas {
    name: String,
    selected: bool,
    pub canvas: Canvas,
}

impl SelectableCanvas {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            selected: false,
            canvas: Canvas::new(name),
        }
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

pub struct CanvasList {
    pub elements: Vec<SelectableCanvas>,
}

impl CanvasList {
    pub fn new() -> Self {
        Self {
            elements: vec![
                SelectableCanvas::new("Canvas 1"),
                SelectableCanvas::new("Canvas 2"),
            ],
        }
    }

    pub fn add_element(&mut self, element: SelectableCanvas) {
        self.elements.push(element);
    }

    pub fn unselect_all(&mut self) {
        for element in &mut self.elements {
            element.selected = false;
        }
    }

    pub fn get_selected(&mut self) -> Option<&mut Canvas> {
        for element in &mut self.elements {
            if element.is_selected() {
                return Some(&mut element.canvas);
            }
        }
        None
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::horizontal()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut to_remove = None;
                    let mut newly_selected = None;
                    
                    for (i, element) in self.elements.iter_mut().enumerate() {
                        ui.spacing_mut().item_spacing.x = 0.0; // Remove horizontal spacing
                        ui.spacing_mut().button_padding.x = 1.0; // Remove padding inside the button
    
                        ui.horizontal(|ui| {
                            // Check for selection
                            if ui.selectable_label(element.selected, &element.name).clicked() {
                                newly_selected = Some(i);
                            }
    
                            // Button to remove the element
                            if ui.button("✖").clicked() {
                                to_remove = Some(i);
                            }
                        });
    
                        *ui.spacing_mut() = ui.style().spacing.clone(); // Reset spacing
                    }
    
                    // Only keep the last selected element selected
                    if let Some(index) = newly_selected {
                        for (i, element) in self.elements.iter_mut().enumerate() {
                            element.selected = i == index;
                        }
                    }
    
                    // Remove the element if the remove button was clicked
                    if let Some(index) = to_remove {
                        self.elements.remove(index);
                    }
                });
            });
    }
}