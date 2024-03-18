use uuid;

use super::gate_list::GhostGate;

#[derive(Debug)]
pub enum CanvasEvent {
    AddGate {
        gate: GhostGate,
        pos: (f32, f32),
        size: (f32, f32),
        id: uuid::Uuid,
    },
    SelectToSpawnGate {
        gate: GhostGate,
        id: uuid::Uuid,
    },
    MoveGate {
        id: uuid::Uuid,
        from_coordinates: (f32, f32),
        to_coordinates: (f32, f32),
    },
    SelectGate {
        id: uuid::Uuid,
    },
    ClickedGateInPut {
        id: uuid::Uuid,
        in_num: usize,
    },
    ClickedGateOutPut {
        id: uuid::Uuid,
        out_num: usize,
    },
    ConnectGates {
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
        in_num: usize,
        out_num: usize,
    },
    RemoveGate {
        id: uuid::Uuid,
    },
}

pub struct CanvasEventQueue {
    pub events: Vec<CanvasEvent>,
    pub current_index: isize,  // Tracks the current position in the event list for undo/redo
    pub changed: bool,  // Tracks whether the Circuit has been changed since the last save
}

impl CanvasEventQueue {
    pub fn new() -> Self {
        CanvasEventQueue {
            events: Vec::new(),
            current_index: -1,  // Start before the first event
            changed: false,
        }
    }

    pub fn get_current_event(&self) -> Option<&CanvasEvent> {
        if self.current_index >= 0 && self.current_index < self.events.len() as isize{
            Some(&self.events[self.current_index as usize])
        } else {
            None
        }
    }    

    pub fn add_event(&mut self, event: CanvasEvent) {
        self.changed = true;
        // If we're not at the end, remove all future events before adding a new one
        // if self.current_index + 1 < self.events.len() as isize {
        //     self.events.truncate(self.current_index as usize + 1);
        // }

        self.events.push(event);
    }

    pub fn undo(&mut self) -> Option<&CanvasEvent> {
        self.changed = true;
        if self.current_index >= 0 {
            let event = &self.events[self.current_index as usize];
            self.current_index -= 1;
            Some(event)  // Return the event to be undone
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&CanvasEvent> {
        self.changed = true;
        if self.current_index + 1 < self.events.len() as isize {
            self.current_index += 1;
            let event = &self.events[self.current_index as usize];
            Some(event)  // Return the event to be redone
        } else {
            None
        }
    }
}
