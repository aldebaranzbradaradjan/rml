//use std::collections::HashMap;
pub mod properties;
pub mod arena;
pub mod draw;

pub use arena::{ArenaNode, ArenaTree, NodeId, PropertyMap, PropertyName, ItemTypeEnum};
pub use properties::{AbstractValue, Property};

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

pub struct RmlEngine {
    arena: ArenaTree,
    properties: HashMap<PropertyId, Property>,
    
    callbacks: HashMap<CallbackId, Callback>,
    bindings: HashMap<PropertyId, Vec<CallbackId>>,
    callbacks_to_eval: Vec<CallbackId>,
}

impl RmlEngine {
    pub fn new() -> Self {
        Self {
            arena: ArenaTree::new(),
            properties: HashMap::new(),
            callbacks: HashMap::new(),
            bindings: HashMap::new(),
            callbacks_to_eval: Vec::new(),
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
    pub fn get_childrens_id(&self, node_id_str: &str) -> Option<Vec<NodeId>> {
        if let Some(node_id) = self.arena.id_to_node_id.get(node_id_str) {
            Some(self.arena.get_childrens_ids(*node_id))
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
                        if property.get() == value {
                            return false;
                        }
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

}

