
use std::{any::Any, error::Error, fmt::format, hash::Hash, path::Path, sync::Arc};
use egui_sdl2_gl::{egui::{self as egui, pos2, Color32, ColorImage, Painter, Pos2, Rect, Rounding, Stroke, TextureHandle, TextureId, TextureOptions}, painter};
use mlua::{Function, Lua, UserData, UserDataMethods, UserDataRef, UserDataRefMut};
use crate::LogicGate;
use super::{canvas::GRID_SPACING, gate_list::GhostGate};
use uuid::Uuid;

#[derive(Clone)]
pub struct GateFiles {
    pub lua: Box<Path>,
    pub json: Option<Box<Path>>,
}

pub struct GateProps {
    pub num_ins: u8,
    pub num_outs: u8,
    pub memory: Option<u8>,
    pub height: Option<u8>,
    pub width: Option<u8>,   
}

impl GateFiles {
    pub fn read_props(&self) -> GateProps {
        // Read the lua file and get the defined properties
        let lua = Lua::new();
        let globals = lua.globals();
        let code = std::fs::read_to_string(&self.lua).unwrap();

        lua.load(&code).exec().unwrap();
        GateProps {
            num_ins: globals.get::<_, u8>("NUM_OF_INS").unwrap(),
            num_outs: globals.get::<_, u8>("NUM_OF_OUTS").unwrap(),
            memory: globals.get::<_, Option<u8>>("MEMORY_SIZE").unwrap(),
            height: globals.get::<_, Option<u8>>("HEIGHT").unwrap(),
            width: globals.get::<_, Option<u8>>("WIDTH").unwrap(),
        }
    }
}

struct VisualBuffer {
    pub buffer: Vec<[u8; 4]>,
    pub size: (u32, u32),
    pub changed: bool,
}

impl UserData for &mut VisualBuffer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_size", |_, this, _: ()| Ok(this.size));
        methods.add_method("get_pixel", |_, this, (x, y): (u32, u32)| Ok(this.buffer[(y * this.size.0 + x) as usize]));
        methods.add_method_mut("test", |_, this, color: (u8, u8, u8, u8)| {
            println!("Test: {:?}", color);
            Ok(())
        });
        methods.add_method_mut("set_pixel", |_, this, (x, y, color): (u32, u32, (u8, u8, u8, u8))| {
            let index = (y * this.size.0 + x) as usize;
            if index < this.buffer.len() {
                this.buffer[index] = (color.0, color.1, color.2, color.3).into();
            }
            this.changed = true;
            Ok(())
        });
        methods.add_method_mut("set_all", |_, this, color: (u8, u8, u8, u8)| {
            for i in 0..this.buffer.len() {
                this.buffer[i] = [color.0, color.1, color.2, color.3];
            }
            this.changed = true;
            Ok(())
        });
    }
}


impl UserData for VisualBuffer { 
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_size", |_, this, _: ()| Ok(this.size));
        methods.add_method("get_pixel", |_, this, (x, y): (u32, u32)| Ok(this.buffer[(y * this.size.0 + x) as usize]));
        methods.add_method_mut("set_pixel", |_, this, (x, y, color): (u32, u32, (u8, u8, u8, u8))| {
            let index = (y * this.size.0 + x) as usize;
            if index < this.buffer.len() {
                this.buffer[index] = (color.0, color.1, color.2, color.3).into();
            }
            this.changed = true;
            Ok(())
        });
        methods.add_meta_method_mut("set_all", |_, this, color: (u8, u8, u8, u8)| {
            for i in 0..this.buffer.len() {
                this.buffer[i] = (color.0, color.1, color.2, color.3).into();
            }
            this.changed = true;
            Ok(())
        });
    }
}

impl VisualBuffer {
    pub fn as_texture(&self) -> egui::ColorImage {
        let width = self.size.0 as usize;
        let height = self.size.1 as usize;
        
        // Convert the buffer ([u8; 4] per pixel) to Vec<egui::Color32>
        let pixels: Vec<egui::Color32> = self.buffer.iter().map(|&color| {
            egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
        }).collect();

        // Create a ColorImage with the converted pixels and dimensions
        egui::ColorImage {
            size: [width, height],
            pixels,
        }
    }
}

struct ImageTexture {
    pub color_image: Arc<egui::ColorImage>,
    pub texture_id: Option<egui::TextureId>,
}

impl ImageTexture {
    fn new(color_image: egui::ColorImage) -> Self {
        let color_image = Arc::new(color_image);
        Self {
            color_image,
            texture_id: None,
        }
    }

    fn update_image_from_buffer(&mut self, buffer: &VisualBuffer) {
        if buffer.changed {
            self.color_image = Arc::new(buffer.as_texture());
            self.texture_id = None;
        }
    }

    fn upload_texture(&mut self, ctx: &egui::Context) {
        if self.texture_id.is_none() {
            let image_data = egui::ImageData::Color(self.color_image.clone());
            let texture_handle = ctx.load_texture("unique_texture_id", image_data, TextureOptions::default());
            self.texture_id = Some(texture_handle.id());
        }
    }

    fn update(&mut self, ctx: &egui::Context, buffer: &VisualBuffer) {
        self.update_image_from_buffer(buffer);
        self.upload_texture(ctx);
    }

    fn texture_id(&self) -> Option<egui::TextureId> {
        self.texture_id
    }
}

pub enum Orientation {
    Up,
    Down,
    Left,
    Right,
}

pub struct DrawableGate {
    gate: Box<dyn LogicGate>,
    pos: (f32, f32),
    size: (f32, f32),
    visual: VisualBuffer,
    texture: ImageTexture,
    pub files: GateFiles,
    pub selected: bool,
    pub drag: (f32, f32),
    pub orientation: Orientation,
    pub id: uuid::Uuid,
}

impl Hash for DrawableGate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for DrawableGate {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl DrawableGate {
    pub fn new(gate: Box<dyn LogicGate>, pos: (f32, f32), size: (f32, f32)) -> Self {
        let lua = Path::new("comps").join(gate.get_name().to_ascii_lowercase() + ".lua");
        let json = Path::new("comps").join(gate.get_name().to_ascii_lowercase() + ".json");

        let files = GateFiles {
            lua: lua.into_boxed_path(),
            json: if json.exists() { Some(json.into_boxed_path()) } else { None },
        };

        let visual = VisualBuffer {
            buffer: vec![[0, 0, 0, 0]; (size.0 * size.1) as usize],
            size: (size.0 as u32, size.1 as u32),
            changed: true,
        };

        Self {
            gate,
            pos,
            size,
            texture: ImageTexture::new(visual.as_texture()),
            visual,
            files,
            selected: false,
            drag: (0.0, 0.0),
            orientation: Orientation::Right,
            id: Uuid::new_v4(),
        }
    }

    pub fn from_ghost(gate: GhostGate, pos: (f32, f32), size: (f32, f32)) -> Self {
        let visual = VisualBuffer {
            buffer: vec![[0, 0, 0, 0]; (size.0 * size.1) as usize],
            size: (size.0 as u32, size.1 as u32),
            changed: true,
        };
        Self {
            gate: gate.gate,
            pos,
            size,
            texture: ImageTexture::new(visual.as_texture()),
            visual,
            files: gate.files,
            selected: false,
            drag: (0.0, 0.0),
            orientation: Orientation::Right,
            id: Uuid::new_v4(),
        }
    }

    fn interaction_logic(&mut self, ui: &mut egui::Ui, gate_rect: egui::Rect, zoom_level: f32) {
        let interact = ui.interact(gate_rect, egui::Id::new(&self), egui::Sense::click_and_drag());

        if interact.clicked() {
            self.selected = !self.selected;
        }

        if self.selected {
            self.selected = true;

            if interact.dragged() {
                self.drag.0 += interact.drag_delta().x / zoom_level;
                self.drag.1 += interact.drag_delta().y / zoom_level;

                if self.drag.0.abs() > GRID_SPACING {
                    self.pos.0 += self.drag.0.signum() * GRID_SPACING;
                    self.drag.0 = 0.0;
                }
                if self.drag.1.abs() > GRID_SPACING {
                    self.pos.1 += self.drag.1.signum() * GRID_SPACING;
                    self.drag.1 = 0.0;
                }
            }
        }
    }

    fn draw_texture(&mut self, painter: &egui::Painter, gate_rect: egui::Rect, ctx: &egui::Context) {
        // Update the texture if visual buffer is changed
        self.texture.update(ctx, &self.visual);

        // If the texture is available, draw it; otherwise, fall back to a default drawing
        if let Some(texture_id) = self.texture.texture_id() {
            // Use the texture to draw the gate's visual representation
            painter.image(texture_id, gate_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),  Color32::WHITE);
        } else {
            panic!("Texture not available");
        }

        // Draw stroke around the gate if it's selected
        if self.selected {
            painter.rect_stroke(gate_rect, egui::Rounding::same(3.0), egui::Stroke::new(5.0, egui::Color32::GRAY));
        }
    }

    pub fn call_lua_gate_draw(&mut self, lua: &Lua) -> mlua::Result<()> {
        lua.scope(|scope| {
            let visual_buff_ref = scope.create_nonstatic_userdata(&mut self.visual)?;
            
            let draw_func: Function = lua.globals().get("Draw")?; // Assuming your Lua function is named "draw_gate"

            draw_func.call::<_, ()>((visual_buff_ref,))?;
            
            Ok(())
        })
    }

    pub fn draw(&mut self,ctx: &egui::Context, ui: &mut egui::Ui, painter: &egui::Painter, pan_offset: egui::Vec2, zoom_level: f32) {
        // Convert the gate's position from a tuple to egui::Vec2 and apply zoom
        let zoom_adjusted_pos = egui::Pos2::new(self.pos.0 * zoom_level, self.pos.1 * zoom_level);
        // Apply the pan offset to the zoom-adjusted position
        let final_pos = zoom_adjusted_pos + pan_offset;
    
        // Define the rectangle for the gate
        let gate_rect = egui::Rect::from_min_max(
            final_pos,
            final_pos + egui::vec2(self.size.0 * zoom_level, self.size.1 * zoom_level),
        );

        self.interaction_logic(ui, gate_rect, zoom_level);

        let lua = Lua::new();
        let code = std::fs::read_to_string(&self.files.lua).unwrap();
        lua.load(&code).exec().unwrap();
        self.call_lua_gate_draw(&lua).unwrap();

        self.draw_texture(painter, gate_rect, ctx);
    }    
}