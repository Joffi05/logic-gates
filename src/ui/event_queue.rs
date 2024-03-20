use core::fmt;
use std::{cell::RefCell, fmt::Formatter, rc::Rc};

use uuid::Uuid;

use super::{drawable_gate::{DrawableGate, InOutPosition}, gate_list::GhostGate};

#[derive(Clone, Debug)]
pub enum GateEvent {
    ClickedOn {
        id: Uuid,
    },
    ClickedIn {
        num: InOutPosition,
        id: Uuid,
    },
    ClickedOut {
        num: InOutPosition,
        id: Uuid,
    },
    MovedGate {
        id: Uuid,
        from: (f32, f32),
        to: (f32, f32),
        start: (f32, f32),        
    },
}

pub enum CanvasEvent {
    SpawnGate {
        gate: GhostGate,
        pos: (f32, f32),
        size: (f32, f32),
    },
    // TODO
    // Für undo muss hier gate info rein
    RemoveSelected,
    AddConnection {
        from_gate: Option<Rc<RefCell<Box<DrawableGate>>>>,
        to_gate: Option<Rc<RefCell<Box<DrawableGate>>>>,
        InputPos: Option<InOutPosition>,
        OutputPos: Option<InOutPosition>,
    },
    PanCanvas {
        from: (f32, f32),
        to: (f32, f32),
    },
    ZoomCanvas {
        from: f32,
        to: f32,
    },
    ClickedCanvas {
        pos: (f32, f32),
    },
    GateEvent(GateEvent),
}

impl std::fmt::Debug for CanvasEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CanvasEvent::SpawnGate { gate, pos, size } => {
                write!(f, "SpawnGate at {:?} with size {:?}", pos, size)
            },
            CanvasEvent::RemoveSelected => {
                write!(f, "RemoveSelected")
            },
            CanvasEvent::AddConnection { from_gate, to_gate, InputPos, OutputPos } => {
                write!(f, "AddConnection at {:?} to {:?}", InputPos, OutputPos)
            },
            CanvasEvent::PanCanvas { from, to } => {
                write!(f, "PanCanvas: from {:?} to {:?}", from, to)
            },
            CanvasEvent::ZoomCanvas { from, to } => {
                write!(f, "ZoomCanvas: from {:?} to {:?}", from, to)
            },
            CanvasEvent::ClickedCanvas { pos } => {
                write!(f, "UnselectAll")
            },
            CanvasEvent::GateEvent(event) => {
                write!(f, "GateEvent: {:?}", event)
            },
        }
    }

}
pub struct EventQueue {
    pub events: Vec<CanvasEvent>,
    pub current_index: isize, // Using isize to easily allow decrement below zero
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue {
            events: Vec::new(),
            current_index: 0, // Start before the first event
        }
    }

    pub fn add_event(&mut self, mut event: CanvasEvent) -> bool {
        // Check if the last event can be mutated to match the new event
        if self.mutate_last_if_same(&mut event) {
            // If the last event cannot be mutated, add the new event to the queue
            self.events.push(event);
            println!("Length: {}", self.events.len());
            // Return true indicating that a new event was added
            return true;
        }

        // Move the current index to the end of the queue
        self.current_index = self.events.len() as isize - 1;
        // Return false indicating that the last event was mutated and no new event was added
        false
    }

    pub fn get_current(&self) -> Option<&CanvasEvent> {
        if self.current_index >= 0 && self.current_index < self.events.len() as isize {
            Some(&self.events[self.current_index as usize])
        } else {
            None
        }
    }

    pub fn get_last_mut(&mut self) -> Option<&mut CanvasEvent> {
        if self.current_index >= 1 && self.current_index - 1 < self.events.len() as isize {
            Some(&mut self.events[(self.current_index - 1) as usize])
        } else {
            None
        }
    }

    fn mutate_last_if_same(&mut self, new_event: &mut CanvasEvent) -> bool {
        if let Some(last_event) = self.get_last_mut() {
            match (last_event, &new_event) {
                (CanvasEvent::PanCanvas { to: last_to, from: last_from }, CanvasEvent::PanCanvas { from, to }) => {
                    // Update the last event with the new coordinates
                    *last_to = *to;
                    false // Indicate that the event was updated and a new one shouldn't be pushed
                },
                (CanvasEvent::GateEvent(GateEvent::MovedGate { to: last_to, from: last_from, .. }), CanvasEvent::GateEvent(GateEvent::MovedGate  { from, to, .. })) => {
                    *last_to = *to;
                    *last_from = *from;
                    false // Similar update logic for MovedGate
                },
                (CanvasEvent::ZoomCanvas { to: last_to, from: last_from}, CanvasEvent::ZoomCanvas { from, to }) => {
                    *last_to = *to;
                    false
                },
                (CanvasEvent::ClickedCanvas { pos: last_to }, CanvasEvent::ClickedCanvas { pos }) => {
                    *last_to = *pos;
                    false
                },
                // Add other cases as necessary...
                _ => true, // Different event types, should push the new event
            }
        } else {
            // No current event, should push the new event
            true
        }
    }


    pub fn advance(&mut self) {
        self.current_index += 1;
    }

    pub fn undo(&mut self) -> Option<&CanvasEvent> {
        if self.current_index >= 0 {
            let event = &self.events[self.current_index as usize];
            self.current_index -= 1; // Move back in time
            Some(event) // Return the event to be undone
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&CanvasEvent> {
        if self.current_index + 1 < self.events.len() as isize {
            self.current_index += 1; // Move forward in time
            let event = &self.events[self.current_index as usize];
            Some(event) // Return the event to be redone
        } else {
            None
        }
    }
}
