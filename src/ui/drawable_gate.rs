
use std::{any::Any, error::Error, fmt::format, hash::Hash, path::Path, sync::Arc};
use egui_sdl2_gl::{egui::{self as egui, pos2, Color32, ColorImage, Painter, Pos2, Rect, Rounding, Stroke, TextureHandle, TextureId, TextureOptions}, painter};
use mlua::{Function, Lua, UserData, UserDataMethods, UserDataRef, UserDataRefMut};
use crate::LogicGate;
use super::{canvas::GRID_SPACING, gate_list::GhostGate};
use uuid::Uuid;

// Describes a position on the border of a gate as number, where an input should go
#[derive(Clone, Debug)]
struct InOutPosition(u16);

impl InOutPosition {
    fn new(pos: u16) -> Self {
        Self(pos)
    }

    fn calc_coord_of_center(&self, rect: egui::Rect) -> (f32, f32) {
        let perimeter = (rect.width() + rect.height()) * 2.0;
        let total_steps = (perimeter / GRID_SPACING).floor() as u16;
        let pos = self.0 % total_steps; // Normalize position to wrap around the rectangle

        let horizontal_steps = (rect.width() / GRID_SPACING).floor() as u16;
        let vertical_steps = (rect.height() / GRID_SPACING).floor() as u16;

        if pos < horizontal_steps {  // Top edge
            return (rect.min.x + pos as f32 * GRID_SPACING, rect.min.y);
        }

        let pos = pos - horizontal_steps;
        if pos < vertical_steps {  // Right edge
            return (rect.max.x, rect.min.y + pos as f32 * GRID_SPACING);
        }

        let pos = pos - vertical_steps;
        if pos < horizontal_steps {  // Bottom edge
            return (rect.max.x - pos as f32 * GRID_SPACING, rect.max.y);
        }

        let pos = pos - horizontal_steps;  // Left edge
        (rect.min.x, rect.max.y - pos as f32 * GRID_SPACING)
    }    
}



#[derive(Clone)]
pub struct GateFiles {
    pub lua: Box<Path>,
    pub json: Option<Box<Path>>,
}

pub struct GateProps {
    pub num_ins: u8,
    pub num_outs: u8,
    pub inputs_pos: Vec<InOutPosition>,
    pub outputs_pos: Vec<InOutPosition>,
    pub memory: Option<u8>,
    pub height: Option<u8>,
    pub width: Option<u8>,   
}

impl GateFiles {
    pub fn read_props(&self) -> Result<GateProps, Box<dyn Error>> {
        // Read the lua file and get the defined properties
        let lua = Lua::new();
        let globals = lua.globals();
        let code = std::fs::read_to_string(&self.lua).unwrap();
        lua.load(&code).exec().unwrap();

        let inputs_pos: Vec<InOutPosition> = globals.get::<_, Vec<u16>>("INPUT_POSITIONS")?
        .iter()
        .map(|&pos| InOutPosition::new(pos))
        .collect();

        let outputs_pos: Vec<InOutPosition> = globals.get::<_, Vec<u16>>("OUTPUT_POSITIONS")?
        .iter()
        .map(|&pos| InOutPosition::new(pos))
        .collect();

        println!("{:?}", inputs_pos);
        println!("{:?}", outputs_pos);

        Ok(GateProps {
            num_ins: globals.get::<_, u8>("NUM_OF_INS")?,
            num_outs: globals.get::<_, u8>("NUM_OF_OUTS")?,
            inputs_pos,
            outputs_pos,
            memory: globals.get::<_, Option<u8>>("MEMORY_SIZE")?,
            height: globals.get::<_, Option<u8>>("HEIGHT")?,
            width: globals.get::<_, Option<u8>>("WIDTH")?,
        })
    }
}

struct VisualBuffer {
    pub buffer: Vec<[u8; 4]>,
    pub size: (u32, u32),
    pub texture: TextureHandle,
    pub changed: bool,
}

impl UserData for &mut VisualBuffer {
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
        
        methods.add_method_mut("set_all", |_, this, color: (u8, u8, u8, u8)| {
            for i in 0..this.buffer.len() {
                this.buffer[i] = [color.0, color.1, color.2, color.3];
            }
            this.changed = true;
            Ok(())
        });
    }
}


impl VisualBuffer {
    pub fn make_texture(&mut self) {
        let width = self.size.0 as usize;
        let height = self.size.1 as usize;
        
        // Convert the buffer ([u8; 4] per pixel) to Vec<egui::Color32>
        let pixels: Vec<egui::Color32> = self.buffer.iter().map(|&color| {
            egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
        }).collect();

        let img = egui::ColorImage {
            size: [width, height],
            pixels,
        };

        self.texture.set(img, TextureOptions::default());
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
    pub fn new(ctx: &egui::Context,gate: Box<dyn LogicGate>, pos: (f32, f32), size: (f32, f32)) -> Self {
        let lua = Path::new("comps").join(gate.get_name().to_ascii_lowercase() + ".lua");
        let json = Path::new("comps").join(gate.get_name().to_ascii_lowercase() + ".json");

        let id = Uuid::new_v4();

        let pixels = vec![[0, 0, 0, 0]; (size.0 * size.1) as usize];
        let pixels_c: Vec<egui::Color32> = pixels.iter().map(|&color| {
            egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
        }).collect();

        let img = egui::ColorImage {
            size: [size.0 as usize, size.1 as usize],
            pixels: pixels_c,
        };

        let files = GateFiles {
            lua: lua.into_boxed_path(),
            json: if json.exists() { Some(json.into_boxed_path()) } else { None },
        };

        let visual = VisualBuffer {
            buffer: pixels,
            size: (size.0 as u32, size.1 as u32),
            texture: ctx.load_texture(format!("gate_texture_{}", id), img, TextureOptions::default()),
            changed: true,
        };

        Self {
            gate,
            pos,
            size,
            visual,
            files,
            selected: false,
            drag: (0.0, 0.0),
            orientation: Orientation::Right,
            id,
        }
    }

    pub fn from_ghost(ctx: &egui::Context, gate: GhostGate, pos: (f32, f32), size: (f32, f32)) -> Self {
        let id = Uuid::new_v4();

        let pixels = vec![[0, 0, 0, 0]; (size.0 * size.1) as usize];
        let pixels_c: Vec<egui::Color32> = pixels.iter().map(|&color| {
            egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
        }).collect();

        let img = egui::ColorImage {
            size: [size.0 as usize, size.1 as usize],
            pixels: pixels_c,
        };

        let visual = VisualBuffer {
            buffer: pixels,
            size: (size.0 as u32, size.1 as u32),
            texture: ctx.load_texture(format!("gate_texture_{}", id), img, TextureOptions::default()),
            changed: true,
        };

        Self {
            gate: gate.gate,
            pos,
            size,
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
        // Update texture if Buffer changed
        if self.visual.changed {
            self.visual.make_texture();
        }

        // draw texture
        painter.image(self.visual.texture.id(), gate_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),  Color32::WHITE);

        // Draw stroke around the gate if it's selected
        if self.selected {
            painter.rect_stroke(gate_rect, egui::Rounding::same(3.0), egui::Stroke::new(5.0, egui::Color32::GRAY));
        }

        // Change later based on computation and or state of the underlying gate
        self.visual.changed = false;
    }

    pub fn call_lua_update_buffer(&mut self, lua: &Lua) -> mlua::Result<()> {
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

        if self.visual.changed {
            let lua = Lua::new();
            let code = std::fs::read_to_string(&self.files.lua).unwrap();
            lua.load(&code).exec().unwrap();
            self.call_lua_update_buffer(&lua).unwrap();
        }
        self.draw_texture(painter, gate_rect, ctx);
    }    
}