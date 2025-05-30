use macroquad::prelude::*;
use std::collections::HashSet;
use crate::{CallbackId, NodeId};

#[derive(Debug, Clone, PartialEq)]
pub enum SystemEvent {
    // Mouse events
    MouseMove { node_id: NodeId, x: f32, y: f32, delta_x: f32, delta_y: f32 },
    MouseWheel { node_id: NodeId, delta_x: f32, delta_y: f32 },
    MouseEnter { node_id: NodeId },
    MouseLeave { node_id: NodeId },
    MouseDown { node_id: NodeId, button: MouseButton, x: f32, y: f32 },
    MouseUp { node_id: NodeId, button: MouseButton, x: f32, y: f32 },
    Click { node_id: NodeId, button: MouseButton, x: f32, y: f32 },
    
    // Window events
    WindowResize { node_id: NodeId, width: f32, height: f32 },
    WindowFocus { node_id: NodeId },
    WindowLostFocus { node_id: NodeId },

    // Keyboard events
    KeyDown { node_id: NodeId, key: KeyCode },
    KeyUp { node_id: NodeId, key: KeyCode },
    KeyPressed { node_id: NodeId, key: KeyCode },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    KeyDown,
    KeyUp,
    KeyPressed,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseWheel,
    MouseEnter,
    MouseLeave,
    Click,
    WindowResize,
    WindowFocus,
    WindowLostFocus,
}

impl SystemEvent {
    pub fn event_type(&self) -> EventType {
        match self {
            SystemEvent::KeyDown { .. } => EventType::KeyDown,
            SystemEvent::KeyUp { .. } => EventType::KeyUp,
            SystemEvent::KeyPressed { .. } => EventType::KeyPressed,
            SystemEvent::MouseDown { .. } => EventType::MouseDown,
            SystemEvent::MouseUp { .. } => EventType::MouseUp,
            SystemEvent::MouseMove { .. } => EventType::MouseMove,
            SystemEvent::MouseWheel { .. } => EventType::MouseWheel,
            SystemEvent::MouseEnter { .. } => EventType::MouseEnter,
            SystemEvent::MouseLeave { .. } => EventType::MouseLeave,
            SystemEvent::Click { .. } => EventType::Click,
            SystemEvent::WindowResize { .. } => EventType::WindowResize,
            SystemEvent::WindowFocus { .. } => EventType::WindowFocus,
            SystemEvent::WindowLostFocus { .. } => EventType::WindowLostFocus,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub callback_id: CallbackId,
    pub event_type: EventType,
    pub node_id: NodeId,
}

#[derive(Debug, Default)]
pub struct EventManager {
    // Single list of all event handlers
    pub handlers: Vec<EventHandler>,
    
    // Current mouse state
    mouse_position: (f32, f32),
    previous_mouse_position: (f32, f32),
    mouse_buttons_pressed: HashSet<MouseButton>,
    
    // Nodes currently under mouse cursor
    pub hovered_nodes: Vec<NodeId>,
    
    // Focused node for keyboard events
    focused_node: Option<NodeId>,
    
    // Window state
    window_size: (f32, f32),
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            mouse_position: (0.0, 0.0),
            previous_mouse_position: (0.0, 0.0),
            mouse_buttons_pressed: HashSet::new(),
            hovered_nodes: Vec::new(),
            focused_node: None,
            window_size: (800.0, 600.0),
        }
    }
    
    pub fn add_event_handler(&mut self, event_type: EventType, node_id: NodeId, callback_id: CallbackId) {
        let handler = EventHandler {
            callback_id,
            event_type,
            node_id,
        };
        self.handlers.push(handler);
    }
    
    pub fn set_focused_node(&mut self, node_id: Option<NodeId>) {
        self.focused_node = node_id;
    }
    
    pub fn get_focused_node(&self) -> Option<NodeId> {
        self.focused_node
    }
    
    pub fn get_mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }
    
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }
    
    pub fn is_node_hovered(&self, node_id: NodeId) -> bool {
        self.hovered_nodes.contains(&node_id)
    }
    
    // pub fn get_handlers_for_event(&self, event_type: &EventType) -> Vec<&EventHandler> {
    //     self.handlers.iter().filter(|h| h.event_type == *event_type).collect()
    // }
    
    pub fn get_handlers_for_node(&self, node_id: NodeId, event_type: &EventType) -> Vec<&EventHandler> {
        self.handlers.iter()
            .filter(|h| h.node_id == node_id && h.event_type == *event_type)
            .collect()
    }
    
    // Update internal state based on macroquad input
    pub fn update_from_macroquad(&mut self) -> Vec<SystemEvent> {
        let mut events = Vec::new();
        
        // Update mouse position
        self.previous_mouse_position = self.mouse_position;
        self.mouse_position = mouse_position();
        
        let delta_x = self.mouse_position.0 - self.previous_mouse_position.0;
        let delta_y = self.mouse_position.1 - self.previous_mouse_position.1;
        
        if delta_x != 0.0 || delta_y != 0.0 {
            events.push(SystemEvent::MouseMove {
                node_id: NodeId::default(), // Placeholder, should be set based on hovered nodes
                x: self.mouse_position.0,
                y: self.mouse_position.1,
                delta_x,
                delta_y,
            });
        }
        
        // Check mouse buttons
        for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            if is_mouse_button_pressed(button) {
                self.mouse_buttons_pressed.insert(button);
                events.push(SystemEvent::MouseDown {
                    node_id: NodeId::default(), // Placeholder, should be set based on hovered nodes
                    button,
                    x: self.mouse_position.0,
                    y: self.mouse_position.1,
                });
            }
            
            if is_mouse_button_released(button) {
                self.mouse_buttons_pressed.remove(&button);
                events.push(SystemEvent::MouseUp {
                    node_id: NodeId::default(), // Placeholder, should be set based on hovered nodes
                    button,
                    x: self.mouse_position.0,
                    y: self.mouse_position.1,
                });
                
                // Generate click event
                events.push(SystemEvent::Click {
                    node_id: NodeId::default(), // Placeholder, should be set based on hovered nodes
                    button,
                    x: self.mouse_position.0,
                    y: self.mouse_position.1,
                });
            }
        }
        
        // Check mouse wheel
        let (wheel_x, wheel_y) = mouse_wheel();
        if wheel_x != 0.0 || wheel_y != 0.0 {
            events.push(SystemEvent::MouseWheel { node_id: NodeId::default(), delta_x: wheel_x, delta_y: wheel_y });
        }
        
        // Check keyboard events
        for key in get_keys_pressed() {
            events.push(SystemEvent::KeyPressed { node_id: NodeId::default(), key });
        }
        
        for key in get_keys_down() {
            events.push(SystemEvent::KeyDown { node_id: NodeId::default(), key });
        }
        
        for key in get_keys_released() {
            events.push(SystemEvent::KeyUp { node_id: NodeId::default(), key });
        }
        
        // Check window resize
        let current_size = (screen_width(), screen_height());
        if current_size != self.window_size {
            self.window_size = current_size;
            events.push(SystemEvent::WindowResize {
                node_id: NodeId::default(), 
                width: current_size.0,
                height: current_size.1,
            });
        }
        
        events
    }
}