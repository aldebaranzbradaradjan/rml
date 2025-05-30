
use std::collections::{HashMap};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use indexmap::IndexSet;

use crate::{Property, PropertyId, AbstractValue};

pub type NodeId = usize;

#[derive(Debug)]
pub struct ArenaTree {
    pub nodes: Vec<ArenaNode>,
    pub id_to_node_id: HashMap<ArenaNodeId, NodeId>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum ItemTypeEnum {
    Node,
    Rectangle,
    Text,
    MouseArea,
}

pub type ArenaNodeId = String;
pub type PropertyName = String;
pub type PropertyMap = HashMap<PropertyName, PropertyId>;

#[derive(Debug)]
pub struct ArenaNode {
    pub id: ArenaNodeId,
    pub node_type: ItemTypeEnum,
    pub properties: PropertyMap,
    pub parent: Option<NodeId>,
    pub children: IndexSet<NodeId>,
}

impl ArenaNode {
    pub fn new(id: ArenaNodeId) -> Self {
        Self {
            id,
            node_type: ItemTypeEnum::Node,
            properties: PropertyMap::new(),
            parent: None,
            children: IndexSet::new(),
        }
    }

    /// Insert a new property into the node's property map, with the given `name` and
    /// `initial_value`. If a property with the same name already exists, it will be
    /// overwritten.
    pub fn add_property(&mut self, name: PropertyName, id: PropertyId) {
        self.properties.insert(name, id);
    }

    pub fn get_property(&self, name: &str) -> Option<usize> {
        self.properties.get(name).copied()
    }
}

impl ToTokens for ItemTypeEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tokenized = match self {
            ItemTypeEnum::Node => quote! { ItemTypeEnum::Node },
            ItemTypeEnum::Rectangle => quote! { ItemTypeEnum::Rectangle },
            ItemTypeEnum::Text => quote! { ItemTypeEnum::Text },
            ItemTypeEnum::MouseArea => quote! { ItemTypeEnum::MouseArea },
        };
        tokenized.to_tokens(tokens);
    }
}

impl ToTokens for Property {

    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tokenized = quote! { Property::new(#self.value.clone()) };
        tokenized.to_tokens(tokens);
    }
}

// Used in macro crate to parse a value
impl ToTokens for AbstractValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let tokenized = match self {
            AbstractValue::Bool(b) => quote! { AbstractValue::Bool(#b) },
            AbstractValue::String(s) => quote! { AbstractValue::String(#s.to_string()) },
            AbstractValue::Number(n) => quote! { AbstractValue::Number(#n) },
            AbstractValue::Array(arr) => {
                let items = arr.iter().map(|item| quote! { #item });
                quote! { AbstractValue::Array(vec![#(#items),*]) }
            }
            AbstractValue::Null => quote! { AbstractValue::Null },
        };
        tokenized.to_tokens(tokens);
    }
}

impl ArenaTree {
    /// Create a new, empty tree
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            id_to_node_id: HashMap::new(),
        }
    }

    /// Add a node to the tree and return its `NodeId`
    pub fn add_node(&mut self, node_type: ItemTypeEnum, id: ArenaNodeId, properties: PropertyMap) -> Option<NodeId> {
        if self.id_to_node_id.contains_key(&id) {
            return None;
        }
        let node_id = self.nodes.len();
        self.nodes.push(ArenaNode {
            id: id.clone(),
            node_type,
            properties,
            parent: None,
            children: IndexSet::new(),
        });
        self.id_to_node_id.insert(id, node_id);
        Some(node_id)
    }

    /// Add a child to a parent node
    pub fn add_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.insert(child_id);
        }
        if let Some(child) = self.nodes.get_mut(child_id) {
            child.parent = Some(parent_id);
        }
    }

    /// Get a node by id
    pub fn get_node_by_id(&self, id: &str) -> Option<&ArenaNode> {
        self.id_to_node_id.get(id).and_then(|&node_id| self.nodes.get(node_id))
    }

    pub fn get_node_mut_by_id(&mut self, id: &str) -> Option<&mut ArenaNode> {
        self.id_to_node_id.get(id).and_then(|&node_id| self.nodes.get_mut(node_id))
    }

    /// Get a node by NodeId
    pub fn get_node(&self, node_id: NodeId) -> Option<&ArenaNode> {
        self.nodes.get(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut ArenaNode> {
        self.nodes.get_mut(node_id)
    }

    /// Get all children ids of a node
    pub fn get_childrens_ids(&self, node_id: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(node_id)
            .map(|node| node.children.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get all children str ids of a node
    pub fn get_childrens_ids_str(&self, node_id: NodeId) -> Vec<ArenaNodeId> {
        self.nodes
            .get(node_id)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|&child_id| self.get_node(child_id).map(|n| n.id.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all children of a node
    pub fn get_children(&self, node_id: NodeId) -> Vec<&ArenaNode> {
        self.nodes
            .get(node_id)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|&child_id| self.get_node(child_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_children_by_id(&self, node_id_str: &str) -> Option<Vec<&ArenaNode>> {
        if let Some(node_id) = self.id_to_node_id.get(node_id_str) {
            Some(self.get_children(*node_id))
        } else {
            None
        }
    }
    
    /// Remove a node by NodeId
    pub fn remove_node(&mut self, node_id: NodeId) {
        let mut parent_node: Option<NodeId> = None;
        let mut children: IndexSet<usize> = IndexSet::new();
        let mut id = String::new();

        if let Some(node) = self.nodes.get(node_id) {
            id = node.id.clone();
            parent_node = node.parent;
            children = node.children.clone();
        }

        // remove node from children of parent
        if let Some(parent_id) = parent_node {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                parent.children.remove(&node_id);
            }
        }
        // remove children
        for &child_id in &children {
            self.remove_node(child_id);
        }
        self.id_to_node_id.remove(&id);
        self.nodes.remove(node_id);
    }
}
