
pub mod properties;
pub mod arena;
pub mod draw;
pub mod events;

use arena::ArenaNodeId;
pub use arena::{ArenaNode, ArenaTree, NodeId, PropertyMap, PropertyName, ItemTypeEnum};
pub use properties::{AbstractValue, Property};
pub use events::{SystemEvent, EventType, EventManager};

use std::{collections::{HashMap}, sync::{Arc}};
use macroquad::color::Color;

pub type CallbackId = usize;
pub type PropertyId = usize;
pub type BindingId = usize;
//pub type Closure = Box<dyn Fn(&mut RmlEngine) + Send + Sync>;
type Callback = Arc<dyn Fn(&mut RmlEngine) + Send + Sync>;

#[macro_export]
macro_rules! get_number {
    ($engine:expr, $node:ident, $prop:ident) => {{
        $engine.get_number_property_of_node(stringify!($node), stringify!($prop), 0.0)
    }};
}

#[macro_export]
macro_rules! set_number {
    ($engine:expr, $node:ident, $prop:ident, $value:expr) => {{
        $engine.set_property_of_node(stringify!($node), stringify!($prop), AbstractValue::Number($value))
    }};
}

#[macro_export]
macro_rules! get_string {
    ($engine:expr, $node:ident, $prop:ident) => {{
        $engine.get_string_property_of_node(stringify!($node), stringify!($prop), "".to_string())
    }};
}

#[macro_export]
macro_rules! set_string {
    ($engine:expr, $node:ident, $prop:ident, $value:expr) => {{
        $engine.set_property_of_node(stringify!($node), stringify!($prop), AbstractValue::String($value))
    }};
}

#[macro_export]
macro_rules! get_bool {
    ($engine:expr, $node:ident, $prop:ident) => {{
        $engine.get_bool_property_of_node(stringify!($node), stringify!($prop), false)
    }};
}

#[macro_export]
macro_rules! set_bool {
    ($engine:expr, $node:ident, $prop:ident, $value:expr) => {{
        $engine.set_property_of_node(stringify!($node), stringify!($prop), AbstractValue::Bool($value))
    }};
}

#[macro_export]
macro_rules! get_color {
    ($engine:expr, $node:ident, $prop:ident) => {{
        $engine.get_color_property_of_node(stringify!($node), stringify!($prop), Color::from_rgba(0, 0, 0, 0))
    }};
}

#[macro_export]
macro_rules! set_color {
    ($engine:expr, $node:ident, $prop:ident, $value:expr) => {{
        $engine.set_property_of_node(stringify!($node), stringify!($prop), AbstractValue::Color($value))
    }};
}

#[macro_export]
macro_rules! get_computed_x {
    ($engine:expr, $node:ident) => {{
        $engine.get_number_property_of_node(stringify!($node), "computed_x", 0.0)
    }};
}

#[macro_export]
macro_rules! get_computed_y {
    ($engine:expr, $node:ident) => {{
        $engine.get_number_property_of_node(stringify!($node), "computed_y", 0.0)
    }};
}

#[macro_export]
macro_rules! get_computed_width {
    ($engine:expr, $node:ident) => {{
        $engine.get_number_property_of_node(stringify!($node), "computed_width", 0.0)
    }};
}

#[macro_export]
macro_rules! get_computed_height {
    ($engine:expr, $node:ident) => {{
        $engine.get_number_property_of_node(stringify!($node), "computed_height", 0.0)
    }};
}


#[macro_export]
macro_rules! get_mouse_wheel_delta_x {
    ($engine:expr) => {{
        if let Some(SystemEvent::MouseWheel { delta_x, .. }) = &$engine.current_event {
            *delta_x
        } else {
            0.0
        }
    }};
}

#[macro_export]
macro_rules! get_mouse_wheel_delta_y {
    ($engine:expr) => {{
        if let Some(SystemEvent::MouseWheel { delta_y, .. }) = &$engine.current_event {
            *delta_y
        } else {
            0.0
        }
    }};
}

#[macro_export]
macro_rules! get_mouse_event_pos {
    ($engine:expr) => {{
        match &$engine.current_event {
            Some(SystemEvent::MouseDown { x, y, .. }) |
            Some(SystemEvent::MouseUp { x, y, .. }) |
            Some(SystemEvent::Click { x, y, .. }) => (*x, *y),
            _ => $engine.get_mouse_position()
        }
    }};
}

#[macro_export]
macro_rules! get_key_event {
    ($engine:expr) => {{
        match &$engine.current_event {
            Some(SystemEvent::KeyDown { key, .. }) |
            Some(SystemEvent::KeyUp { key, .. }) |
            Some(SystemEvent::KeyPressed { key, .. }) => Some(*key),
            _ => None
        }
    }};
}

#[macro_export]
macro_rules! emit {
    ($engine:expr, $node:ident, $signal:ident) => {{
        // Emit a signal by setting the signal property to trigger callbacks
        // We use a toggle mechanism to ensure the signal always triggers
        let current_value = $engine.get_bool_property_of_node(stringify!($node), stringify!($signal), false);
        $engine.set_property_of_node(stringify!($node), stringify!($signal), AbstractValue::Bool(!current_value))
    }};
}

#[macro_export]
macro_rules! get_value {
    ($engine:expr, $node:ident, $prop:ident) => {{
        // Get the raw AbstractValue - let the context determine how to use it
        if let Some(prop_id) = $engine.get_property_id_of_node(stringify!($node), stringify!($prop)) {
            if let Some(property) = $engine.get_property(prop_id) {
                property.get().clone()
            } else {
                AbstractValue::Null
            }
        } else {
            AbstractValue::Null
        }
    }};
}

pub fn lighter_color(color: Color, amount: f32) -> Color {
    let mut color = color;
    color.r += amount;
    color.g += amount;
    color.b += amount;
    color
}

pub fn darker_color(color: Color, amount: f32) -> Color {
    let mut color = color;
    color.r -= amount;
    color.g -= amount;
    color.b -= amount;
    color
}

pub fn decompose_color_string(color_string: &str) -> Color {
    //"rgba(0.4, 0.9, 0.7, 1.0)"
    // remove rgba( and )
    // split by comma
    // parse each part as f32
    let s = color_string.trim_start_matches("rgba(").trim_end_matches(")");
    let parts: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
    let r = parts[0].parse::<f32>().unwrap();
    let g = parts[1].parse::<f32>().unwrap();
    let b = parts[2].parse::<f32>().unwrap();
    let a = parts[3].parse::<f32>().unwrap();

    //Color::from_rgba(r as u8, g as u8, b as u8, a as u8)
    Color::new(r, g, b, a)
}

pub struct RmlEngine {
    arena: ArenaTree,
    properties: HashMap<PropertyId, Property>,
    
    callbacks: HashMap<CallbackId, Callback>,
    bindings: HashMap<PropertyId, Vec<CallbackId>>,
    callbacks_to_eval: Vec<CallbackId>,
    
    event_manager: EventManager,
    pub current_event: Option<SystemEvent>,
    pub current_event_consumed: bool,
}

impl RmlEngine {
    pub fn new() -> Self {
        Self {
            arena: ArenaTree::new(),
            properties: HashMap::new(),
            callbacks: HashMap::new(),
            bindings: HashMap::new(),
            callbacks_to_eval: Vec::new(),
            event_manager: EventManager::new(),
            current_event: None,
            current_event_consumed: false,
        }
    }

    pub fn get_arena(&self) -> &ArenaTree {
        &self.arena
    }

    pub fn get_arena_mut(&mut self) -> &mut ArenaTree {
        &mut self.arena
    }

    // arena tree methods
    pub fn add_node(&mut self, id: String, node_type: ItemTypeEnum, properties: PropertyMap) -> Option<NodeId> {
        self.arena.add_node(node_type, id, properties)
    }

    pub fn add_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        self.arena.add_child(parent_id, child_id);
    }

    pub fn get_node_by_id(&self, id: &str) -> Option<&ArenaNode> {
        self.arena.get_node_by_id(id)
    }

    pub fn get_node_mut_by_id(&mut self, id: &str) -> Option<&mut ArenaNode> {
        self.arena.get_node_mut_by_id(id)
    }

    pub fn get_node(&self, node_id: NodeId) -> Option<&ArenaNode> {
        self.arena.get_node(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut ArenaNode> {
        self.arena.get_node_mut(node_id)
    }

    pub fn get_children(&self, node_id: NodeId) -> Vec<&ArenaNode> {
        self.arena.get_children(node_id)
    }

    pub fn get_children_by_id(&self, node_id_str: &str) -> Option<Vec<&ArenaNode>> {
        self.arena.get_children_by_id(node_id_str)
    }

    pub fn get_childrens_ids(&self, node_id_str: &str) -> Vec<NodeId> {
        if let Some(node_id) = self.arena.id_to_node_id.get(node_id_str) {
            return self.arena.get_childrens_ids(*node_id);
        }
        else {
            Vec::new()
        }
    }

    pub fn get_children_str_ids(&self, node_id: NodeId) -> Vec<ArenaNodeId> {
        self.arena.get_childrens_ids_str(node_id)
    }

    pub fn get_children_str_ids_by_id(&self, node_id_str: &str) -> Option<Vec<ArenaNodeId>> {
        if let Some(node_id) = self.arena.id_to_node_id.get(node_id_str) {
            Some(self.arena.get_childrens_ids_str(*node_id))
        } else {
            None
        }
    }

    pub fn get_childrens_id(&self, node_id_str: &str) -> Option<Vec<NodeId>> {
        if let Some(node_id) = self.arena.id_to_node_id.get(node_id_str) {
            Some(self.arena.get_childrens_ids(*node_id))
        } else {
            None
        }
    }

    pub fn get_parent_by_id(&self, node_id_str: &str) -> Option<&ArenaNode> {
        if let Some(parent_id) = self.get_parent_id(node_id_str) {
            self.arena.get_node(parent_id)
        } else {
            None
        }
    }

    pub fn get_parent_id(&self, node_id_str: &str) -> Option<NodeId> {
        if let Some(node_id) = self.arena.id_to_node_id.get(node_id_str) {
            self.arena.get_node(*node_id).and_then(|node| node.parent)
        } else {
            None
        }
    }
    
    pub fn remove_node(&mut self, node_id: NodeId) {
        self.arena.remove_node(node_id);
    }
    pub fn get_node_id(&self, id: &str) -> Option<NodeId> {
        self.arena.id_to_node_id.get(id).copied()
    }
    pub fn add_property_to_node(&mut self, node_id: NodeId, name: PropertyName, property_id: PropertyId) {
        if let Some(node) = self.arena.get_node_mut(node_id) {
            node.add_property(name, property_id);
        }
    }

    pub fn get_property_of_node<T>(
        &self,
        node_name: &str,
        property_name: &str,
        default_value: T,
        convert: impl Fn(&AbstractValue) -> Option<T>,
    ) -> T {
        self.arena
            .id_to_node_id
            .get(node_name)
            .and_then(|node_id| self.arena.get_node(*node_id))
            .and_then(|node| node.get_property(property_name))
            .and_then(|prop_id| self.get_property(prop_id))
            .and_then(|property| convert(&property.value))
            .unwrap_or(default_value)
    }

    pub fn get_property_id_of_node(&self, node_name: &str, property_name: &str) -> Option<PropertyId> {
        self.arena
            .id_to_node_id
            .get(node_name)
            .and_then(|node_id| self.arena.get_node(*node_id))
            .and_then(|node| node.get_property(property_name))
    }

    pub fn get_number_property_of_node(&self, node_name: &str, property_name: &str, default_value: f32) -> f32 {
        self.get_property_of_node(node_name, property_name, default_value, |v| v.to_number().map(|n| n as f32))
    }

    pub fn get_string_property_of_node(&self, node_name: &str, property_name: &str, default_value: String) -> String {
        self.get_property_of_node(node_name, property_name, default_value, |v| Some(v.to_string()))
    }

    pub fn get_bool_property_of_node(&self, node_name: &str, property_name: &str, default_value: bool) -> bool {
        self.get_property_of_node(node_name, property_name, default_value, |v| v.to_bool())
    }
    
    pub fn get_color_property_of_node(&self, node_name: &str, property_name: &str, default_value: Color) -> Color {
        self.get_property_of_node(node_name, property_name, default_value, |v| v.to_color())
    }

    pub fn set_property_of_node(&mut self, node_name: &str, property_name: &str, value: AbstractValue) -> bool {
        if let Some(node_id) = self.arena.id_to_node_id.get(node_name) {
            if let Some(node) = self.arena.get_node_mut(*node_id) {
                if let Some(property_id) = node.get_property(property_name) {
                    if let Some(property) = self.get_property_mut(property_id) {
                        // test if value changed
                        if property.get() == value { return false; }
                        property.set(value);

                        if let Some(callback_ids) = self.bindings.get(&property_id) {
                            for &cb_id in callback_ids {
                                self.callbacks_to_eval.push(cb_id);
                            }
                        }

                        true;
                    }
                }
            }
        }
        false
    }

    pub fn add_property(&mut self, property: Property) -> PropertyId {
        let id = self.properties.len();
        self.properties.insert(id, property);
        id
    }

    pub fn get_property(&self, id: PropertyId) -> Option<&Property> {
        self.properties.get(&id)
    }

    pub fn get_property_mut(&mut self, id: PropertyId) -> Option<&mut Property> {
        self.properties.get_mut(&id)
    }

    pub fn remove_property(&mut self, id: PropertyId) {
        self.properties.remove(&id);
    }

    pub fn get_property_by_name(&self, node_id: NodeId, name: &str) -> Option<&Property> {
        if let Some(node) = self.arena.get_node(node_id) {
            if let Some(property_id) = node.get_property(name) {
                return self.get_property(property_id);
            }
        }
        None
    }

    pub fn get_property_by_name_mut(&mut self, node_id: NodeId, name: &str) -> Option<&mut Property> {
        if let Some(node) = self.arena.get_node(node_id) {
            if let Some(property_id) = node.get_property(name) {
                return self.get_property_mut(property_id);
            }
        }
        None
    }

    pub fn add_callback<F>(&mut self, callback: F) -> CallbackId
    where
        F: Fn(&mut RmlEngine) + Send + Sync + 'static,
    {
        let id = self.callbacks.len();
        self.callbacks.insert(id, Arc::new(callback));
        id
    }

    pub fn bind_property_to_callback(&mut self, prop_id: PropertyId, callback_id: CallbackId) {
        self.bindings
            .entry(prop_id)
            .or_default()
            .push(callback_id);
    }

    pub fn bind_node_property_to_callback(&mut self, node_name: &str, property_name: &str, callback_id: CallbackId) {
        if let Some(node_id) = self.arena.id_to_node_id.get(node_name) {
            if let Some(node) = self.arena.get_node(*node_id) {
                if let Some(prop_id) = node.get_property(property_name) {
                    self.bind_property_to_callback(prop_id, callback_id);
                }
            }
        }
    }

    pub fn run_callbacks(&mut self) {
        let to_eval = std::mem::take(&mut self.callbacks_to_eval);

        // clone les callbacks à exécuter
        let callbacks_to_run: Vec<_> = to_eval
            .into_iter()
            .filter_map(|cb_id| self.callbacks.get(&cb_id).cloned().map(|cb| (cb_id, cb)))
            .collect();

        for (_cb_id, callback) in callbacks_to_run {
            callback(self);
        }
    }

    pub fn get_node_type(&self, node_id: &str) -> Option<ItemTypeEnum> {
        self.arena.get_node_by_id(node_id).map(|node| node.node_type.clone())
    }

    // Event system methods
    pub fn add_event_handler(&mut self, event_type: EventType, node_id_str: &str, callback_id: CallbackId) {
        if let Some(node_id) = self.get_node_id(node_id_str) {
            self.event_manager.add_event_handler(event_type, node_id, callback_id);
        }
    }
    
    pub fn get_event_manager(&self) -> &EventManager {
        &self.event_manager
    }
    
    pub fn get_event_manager_mut(&mut self) -> &mut EventManager {
        &mut self.event_manager
    }
    
    pub fn set_focused_node(&mut self, node_id_str: &str) {
        let node_id = self.get_node_id(node_id_str);
        self.event_manager.set_focused_node(node_id);
    }
    
    pub fn process_events(&mut self) -> Vec<SystemEvent> {
        // Update from macroquad input
        let events = self.event_manager.update_from_macroquad();
        let hovered_nodes = self.get_mouse_area_nodes_under_mouse();
        let mouse_area_nodes = self.get_mouse_area_nodes();
        let focused_node = self.event_manager.get_focused_node();
        let mut current_hovered_nodes = Vec::new();
        self.current_event_consumed = false;

        // we will refine events before handling them
        // Some events can be consumed, some affects hovered only, mousearea only or focused only, and some are globals

        // Handle mouse events
        // check for enter event
        for node in hovered_nodes.clone() {
            current_hovered_nodes.push(node);
            if !self.event_manager.is_node_hovered(node) {
                // If the node is not hovered, we trigger MouseEnter
                self.handle_system_event(&SystemEvent::MouseEnter { node_id: node });
            }
            if let Some(consume) = self.get_property_by_name(node, "consume_mouse_enter") {
                self.current_event_consumed = consume.get().to_bool().unwrap_or(false);
                if self.current_event_consumed { break; }
            }
        }

        for event in &events {
            match event {
                // global events
                SystemEvent::WindowResize { .. } |SystemEvent::WindowFocus { .. } | SystemEvent::WindowLostFocus { .. } => {
                    self.handle_system_event(event);
                }

                // key events
                SystemEvent::KeyUp { node_id: _, key } => {
                    if let Some(node_id) = focused_node {
                        self.handle_system_event(&SystemEvent::KeyUp { node_id: node_id, key: *key });
                    }
                }
                SystemEvent::KeyDown { node_id: _, key } => {
                    if let Some(node_id) = focused_node {
                        self.handle_system_event(&SystemEvent::KeyDown { node_id: node_id, key: *key });
                    }
                    
                }
                SystemEvent::KeyPressed { node_id: _, key } => {
                    if let Some(node_id) = focused_node {
                        self.handle_system_event(&SystemEvent::KeyPressed { node_id: node_id, key: *key });
                    }
                }

                // mouse events
                SystemEvent::Click { node_id: _, x, y, button } => {
                    for node in &hovered_nodes {
                        self.handle_system_event(&SystemEvent::Click { node_id: *node, x: *x, y: *y, button: *button });
                        if let Some(consume) = self.get_property_by_name(*node, "consume_mouse_click") {
                            self.current_event_consumed = consume.get().to_bool().unwrap_or(false);
                            if self.current_event_consumed { break; }
                        }
                    }
                }
                SystemEvent::MouseDown { node_id: _, x, y, button } => {
                    for node in &hovered_nodes {
                        self.handle_system_event(&SystemEvent::MouseDown { node_id: *node, x: *x, y: *y, button: *button });
                        if let Some(consume) = self.get_property_by_name(*node, "consume_mouse_down") {
                            self.current_event_consumed = consume.get().to_bool().unwrap_or(false);
                            if self.current_event_consumed { break; }
                        }
                    }
                }
                SystemEvent::MouseUp { node_id: _, x, y, button } => {
                    for node in &hovered_nodes {
                        self.handle_system_event(&SystemEvent::MouseUp { node_id: *node, x: *x, y: *y, button: *button });
                        if let Some(consume) = self.get_property_by_name(*node, "consume_mouse_up") {
                            self.current_event_consumed = consume.get().to_bool().unwrap_or(false);
                            if self.current_event_consumed { break; }
                        }
                    }
                }
                SystemEvent::MouseMove { node_id: _, x, y, delta_x, delta_y } => {
                    for node in &mouse_area_nodes {
                        self.handle_system_event(&SystemEvent::MouseMove { node_id: *node, x: *x, y: *y, delta_x: *delta_x, delta_y: *delta_y });
                        if let Some(consume) = self.get_property_by_name(*node, "consume_mouse_move") {
                            self.current_event_consumed = consume.get().to_bool().unwrap_or(false);
                            if self.current_event_consumed { break; }
                        }
                    }
                }
                SystemEvent::MouseWheel { node_id: _, delta_x, delta_y } => {
                    for node in &mouse_area_nodes {
                        self.handle_system_event(&SystemEvent::MouseWheel { node_id: *node, delta_x: *delta_x, delta_y: *delta_y });
                        if let Some(consume) = self.get_property_by_name(*node, "consume_mouse_wheel") {
                            self.current_event_consumed = consume.get().to_bool().unwrap_or(false);
                            if self.current_event_consumed { break; }
                        }
                    }
                }
                _ => {}
            }
        }

        // Check for nodes that are no longer hovered
        let previously_hovered_nodes = self.event_manager.hovered_nodes.clone();
        self.current_event_consumed = false;

        for &node_id in &previously_hovered_nodes {
            if !current_hovered_nodes.contains(&node_id) {
                // If the node was previously hovered but is no longer, trigger MouseLeave
                self.handle_system_event(&SystemEvent::MouseLeave { node_id });
            }
            if let Some(consume) = self.get_property_by_name(node_id, "consume_mouse_leave") {
                self.current_event_consumed = consume.get().to_bool().unwrap_or(false);
                if self.current_event_consumed { break; }
            }
        }

        // Update hovered nodes
        self.event_manager.hovered_nodes = current_hovered_nodes;
        
        // Run any property change callbacks that might have been triggered by a property set
        self.run_callbacks();
        
        events
    }
    
    fn handle_system_event(&mut self, event: &SystemEvent) {
        let event_type = event.event_type();
        let mut callbacks_with_event = Vec::new();
        
        match event {
            // node related events
            SystemEvent::MouseWheel { node_id, .. }
            | SystemEvent::MouseMove { node_id, .. } 
            | SystemEvent::MouseDown { node_id, .. }
            | SystemEvent::MouseUp { node_id, .. }
            | SystemEvent::Click { node_id, .. }
            | SystemEvent::MouseEnter { node_id }
            | SystemEvent::MouseLeave { node_id }
            | SystemEvent::KeyDown { node_id, .. }
            | SystemEvent::KeyUp { node_id, .. }
            | SystemEvent::KeyPressed { node_id, .. } => {
                let handlers = self.event_manager.get_handlers_for_node(*node_id, &event_type);
                for handler in handlers {
                    callbacks_with_event.push((handler.callback_id, event.clone()));
                }
            },
            // globals events
            SystemEvent::WindowResize { .. }
            | SystemEvent::WindowFocus { .. }
            | SystemEvent::WindowLostFocus { .. } => {
                let handlers = self.event_manager.get_handlers_for_event(&event_type);
                for handler in handlers {
                    callbacks_with_event.push((handler.callback_id, event.clone()));
                }
            }
        }

        // Execute callbacks immediately with their specific event (at the difference of other events that are processed in process_events)
        for (callback_id, event) in callbacks_with_event {
            if let Some(callback) = self.callbacks.get(&callback_id).cloned() {
                // Set the specific event for this callback
                self.current_event = Some(event);
                callback(self);
                // Clear immediately after callback execution
                self.current_event = None;
                
            }
        }
    }
    
    fn get_mouse_area_nodes_under_mouse(&self) -> Vec<NodeId> {
        let mouse_pos = self.event_manager.get_mouse_position();
        self.get_mouse_area_nodes_at_position(mouse_pos.0, mouse_pos.1)
    }
    

    fn get_mouse_area_nodes_at_position(&self, x: f32, y: f32) -> Vec<NodeId> {
        let mut nodes = Vec::new();
        
        // Only check MouseArea nodes for mouse events
        for node in &self.arena.nodes {
            if node.node_type == ItemTypeEnum::MouseArea {
                if let Some(node_id) = self.arena.id_to_node_id.get(&node.id) {
                    if self.is_point_inside_node(*node_id, x, y) {
                        nodes.push(*node_id);
                    }
                }
            }
        }

        nodes.reverse(); // Reverse to ensure topmost nodes are checked first
        // like the nodes are created in a top-down manner, the last created node is the topmost one
        // and any node before a node in the list is below it visually 
        nodes
    }

    fn get_mouse_area_nodes(&self) -> Vec<NodeId> {
        let mut nodes = Vec::new();
        
        // Only check MouseArea nodes for mouse events
        for node in &self.arena.nodes {
            if node.node_type == ItemTypeEnum::MouseArea {
                if let Some(node_id) = self.arena.id_to_node_id.get(&node.id) {
                    nodes.push(*node_id);
                }
            }
        }

        nodes.reverse(); // Reverse to ensure topmost nodes are checked first
        // like the nodes are created in a top-down manner, the last created node is the topmost one
        // and any node before a node in the list is below it visually 
        nodes
    }
    
    fn is_point_inside_node(&self, node_id: NodeId, x: f32, y: f32) -> bool {
        // Use pre-computed absolute geometry from draw phase
        if let Some(node) = self.arena.get_node(node_id) {
            let computed_abs_x = self.get_number_property_of_node(&node.id, "computed_x", 0.0);
            let computed_abs_y = self.get_number_property_of_node(&node.id, "computed_y", 0.0);
            let computed_width = self.get_number_property_of_node(&node.id, "computed_width", 0.0);
            let computed_height = self.get_number_property_of_node(&node.id, "computed_height", 0.0);
            
            // Now we can directly compare with mouse coordinates (both in window coordinates)
            x >= computed_abs_x && x <= computed_abs_x + computed_width && 
            y >= computed_abs_y && y <= computed_abs_y + computed_height
        } else {
            false
        }
    }

    pub fn get_mouse_position(&self) -> (f32, f32) {
        self.event_manager.get_mouse_position()
    }

}

pub mod prelude {
    pub use macroquad::prelude::*;
    pub use std::collections::HashMap;

    pub use super::{
        RmlEngine,
        Property,
        AbstractValue,
        get_value,
        get_bool,
        set_bool,
        get_number,
        set_number,
        get_string,
        set_string,
        darker_color,
        lighter_color,
        get_color,
        emit,
        EventType,
        ItemTypeEnum,
        decompose_color_string,
        get_key_event,
        SystemEvent,
        NodeId
    };
}