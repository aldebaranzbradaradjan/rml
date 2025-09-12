
use quote::{format_ident};
use syn::{Ident, Lit};
use rml_core::{AbstractValue};
use std::collections::HashMap;

// struct to represent a property key
// It can be a simple identifier or a composed one (base.field)
#[derive(Debug, Clone)]
pub enum PropertyKey {
    Simple(Ident),
    Composed { 
        base: Ident, 
        field: Ident 
    },
    Signal(Ident),
}

// struct to represent an import statement
#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub path: String,
    pub alias: Option<String>,
}

#[derive(Clone)]
pub struct ComponentDefinition {
    pub name: String,
    pub path: String,
}

impl PropertyKey {
    pub fn to_string(&self) -> String {
        match self {
            PropertyKey::Simple(ident) => ident.to_string(),
            PropertyKey::Composed { base, field } => format!("{}_{}", base, field),
            PropertyKey::Signal(ident) => ident.to_string(),
        }
    }
    
    pub fn to_ident(&self) -> Ident {
        format_ident!("{}", self.to_string())
    }
    
    pub fn is_signal(&self) -> bool {
        matches!(self, PropertyKey::Signal(_))
    }
}


#[derive(Clone)]
pub enum Value {
    Lit(Lit),
    Ident(Ident),
    Block(syn::Block),
}

pub fn lit_to_string(literal: &Lit) -> Option<String> {
    match literal {
        Lit::Str(lit_str) => Some(lit_str.value()),
        Lit::ByteStr(lit_byte_str) => {
            Some(String::from_utf8_lossy(&lit_byte_str.value()).to_string())
        }
        Lit::Char(lit_char) => Some(lit_char.value().to_string()),
        Lit::Int(lit_int) => Some(lit_int.base10_digits().to_string()),
        Lit::Float(lit_float) => Some(lit_float.base10_digits().to_string()),
        Lit::Bool(lit_bool) => Some(lit_bool.value.to_string()),
        Lit::Byte(lit_byte) => Some(format!("{:?}", lit_byte.value())),
        _ => None,
    }
}

pub fn value_to_abstract_value(value: &Value) -> AbstractValue {
    match value {
        Value::Lit(literal) => match literal {
            Lit::Str(lit_str) => AbstractValue::String(lit_str.value()),
            Lit::ByteStr(lit_byte_str) => {
                AbstractValue::String(String::from_utf8_lossy(&lit_byte_str.value()).to_string())
            }
            Lit::Char(lit_char) => AbstractValue::String(lit_char.value().to_string()),
            Lit::Int(lit_int) => {
                let number: f32 = lit_int.base10_parse().unwrap_or_default();
                AbstractValue::Number(number)
            }
            Lit::Float(lit_float) => {
                let number: f32 = lit_float.base10_parse().unwrap_or_default();
                AbstractValue::Number(number)
            }
            Lit::Bool(lit_bool) => AbstractValue::Bool(lit_bool.value),
            Lit::Byte(lit_byte) => AbstractValue::Number(lit_byte.value() as f32),
            _ => AbstractValue::Null,
        },
        Value::Ident(ident) => AbstractValue::String(ident.to_string()),
        Value::Block(_block) => AbstractValue::Null,
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Lit(literal) => lit_to_string(literal).unwrap_or_default(),
            Value::Ident(ident) => ident.to_string(),
            Value::Block(_) => "<block>".to_string(),
        }
    }
}

/// Struct to parse a Node
#[derive(Clone)]
pub struct RmlNode {
    pub _ident: String,
    pub properties: Vec<(PropertyKey, Value)>,
    pub children: Vec<RmlNode>,
    pub functions: Vec<syn::ItemFn>,
}

/// Main RML parser that includes imports
pub struct RmlParser {
    pub components: HashMap<String, ComponentDefinition>,
    pub root_node: RmlNode,
}


pub type GenResult = (String, proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream);