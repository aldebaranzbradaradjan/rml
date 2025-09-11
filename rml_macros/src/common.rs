use proc_macro2::Span;
use quote::format_ident;
use syn::{parse_macro_input, Ident, Lit, Token, Expr, ExprPath, Member, LitStr};
use syn::parse::ParseStream;
use syn::parse::{Parse, ParseBuffer};
use rml_core::AbstractValue;

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

/// Struct to parse a Node
#[derive(Clone)]
pub struct RmlNode {
    pub _ident: String,
    pub properties: Vec<(PropertyKey, Value)>,
    pub children: Vec<RmlNode>,
    pub functions: Vec<syn::ItemFn>,
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

type GenResult = (String, proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream);

impl Parse for RmlNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        // node name can be a sigle Ident or Ident . Ident
        let _ident: Ident = input.parse()?;
        let mut _ident = _ident.to_string();    
        if input.peek(Token![::]) {
            input.parse::<Token![::]>()?;
            let second_ident: Ident = input.parse()?;
            _ident = format!("{}::{}", _ident, second_ident.to_string());
        }


        let content;
        syn::braced!(content in input);

        let mut properties = Vec::new();
        let mut children = Vec::new();
        let mut functions = Vec::new();

        while !content.is_empty() {
            if content.peek(Token![fn]) {
                let item: syn::Item = content.parse()?;
                if let syn::Item::Fn(func) = item {
                    functions.push(func);
                } else {
                    return Err(content.error("Expected function"));
                }
            }
            else {
                // Check for signal declaration first
                if content.peek(Ident) {
                    let fork = content.fork();
                    if let Ok(signal_keyword) = fork.parse::<Ident>() {
                        if signal_keyword == "signal" {
                            // parse signal declaration: signal identifier
                            content.parse::<Ident>()?; // consume "signal" string
                            let signal_name: Ident = content.parse()?;
                            content.parse::<Token![,]>().ok();
                            // add signal as a special property
                            properties.push((PropertyKey::Signal(signal_name), Value::Ident(Ident::new("signal", Span::call_site()))));
                            continue;
                        }
                    }
                }
                
                // will try to parse the property first
                let fork = content.fork();
                
                // try to parse a property key
                if let Ok(_key) = parse_property_key(&fork) {
                    if fork.peek(Token![:]) {
                        // assume it's a property
                        let key = parse_property_key(&content)?;
                        content.parse::<Token![:]>()?;
                        let value = property_parse(&content)?;
                        content.parse::<Token![,]>().ok();
                        properties.push((key, value));
                        continue;
                    }
                }
                
                // if we are here, it's not a property; try to parse a child node
                if content.peek(Ident) {
                    let child: RmlNode = content.parse()?;
                    functions.extend(child.functions.clone());
                    children.push(child);
                } else {
                    return Err(content.error("Unexpected token"));
                }
            }
        }

        Ok(Self {
            _ident,
            properties,
            children,
            functions,
        })
    }
}


/// Extrait l'ID d'un nœud à partir de ses propriétés
pub fn extract_node_id(node: &RmlNode) -> Option<String> {
    node.properties.iter().find_map(|(k, v)| {
        if k.to_string() == "id" { 
            Some(v.to_string()) 
        } else { 
            None 
        }
    })
}



/// Applique les propriétés d'une instance sur un composant résolu
pub fn apply_instance_properties(
    resolved_component: &mut RmlNode,
    instance_properties: &[(PropertyKey, Value)]
) {
    for (prop_key, prop_value) in instance_properties {
        if let Some(existing_prop) = resolved_component.properties.iter_mut()
            .find(|(k, _)| k.to_string() == prop_key.to_string()) {
            // Écrase la propriété existante
            existing_prop.1 = prop_value.clone();
        } else {
            // Ajoute une nouvelle propriété
            resolved_component.properties.push((prop_key.clone(), prop_value.clone()));
        }
    }
}

pub fn inject_engine_in_block(mut block: syn::Block, initializer: bool) -> syn::Block {
    use syn::{Expr, Stmt};

    block.stmts = block
        .stmts
        .into_iter()
        .map(|stmt| match stmt {
            Stmt::Expr(expr, semi_opt) => {
                let expr = match expr {
                    Expr::Call(mut call) => {
                        let has_engine = call.args.iter().any(|arg| {
                            matches!(arg, Expr::Path(p) if p.path.is_ident("engine"))
                        });

                        if !has_engine {
                            if initializer {
                                let engine_expr: Expr = syn::parse_quote!(&mut engine);
                                call.args.insert(0, engine_expr);
                            } else {
                                let engine_expr: Expr = syn::parse_quote!(engine);
                                call.args.insert(0, engine_expr);
                            }
                        }

                        Expr::Call(call)
                    }
                    other => other,
                };
                Stmt::Expr(expr, semi_opt)
            }
            other => other,
        })
        .collect();

    block
}

// Function to parse a property key; it can be a simple identifier or a composed one (base.field)
pub fn parse_property_key(input: ParseStream) -> syn::Result<PropertyKey> {
    let expr: Expr = input.parse()?;
    
    match expr {
        // it's a simple identifier : `width`
        Expr::Path(ExprPath { path, .. }) if path.segments.len() == 1 => {
            let ident = path.segments.first().unwrap().ident.clone();
            Ok(PropertyKey::Simple(ident))
        }
        
        // it's a composed identifier (fake field access) : `layout.fill_width`
        Expr::Field(field_expr) => {
            if let Expr::Path(ExprPath { path, .. }) = &*field_expr.base {
                if path.segments.len() == 1 {
                    let base = path.segments.first().unwrap().ident.clone();
                    
                    if let Member::Named(field_ident) = &field_expr.member {
                        return Ok(PropertyKey::Composed { 
                            base, 
                            field: field_ident.clone() 
                        });
                    }
                }
            }
            Err(input.error("Invalid field access in property"))
        }
        
        _ => Err(input.error("Expected simple identifier or field access (base.field)"))
    }
}



pub fn property_parse(content: &ParseBuffer) -> Result<Value, syn::Error> {
    let value: Value;
    if content.peek(Lit) {
        value = Value::Lit(content.parse()?);
    } else if content.peek(Ident) {
        if content.peek(Ident) && content.peek2(Token![|]) {
            // handle composed values with |
            let mut composed_value = String::new();
            while !content.peek(Token![|]) {
                let ident: Ident = content.parse()?;
                composed_value.push_str(&ident.to_string());
                composed_value.push_str("__");

                if content.peek(Token![|]) {
                    content.parse::<Token![|]>()?;
                }
                if content.peek(Token![;]) || content.peek(Token![,]) {
                    break;
                }
                if content.peek(Ident) && content.peek2(Token![:]) {
                    break;
                }
                if content.peek(Ident) && content.peek2(Token![.]) {
                    break;
                }
            }
            value = Value::Ident(Ident::new(&composed_value, Span::call_site()));
        } else {
            value = Value::Ident(content.parse()?);
        }
    } else if content.peek(syn::token::Brace) {
        let block: syn::Block = content.parse()?;
        value = Value::Block(block);
    } else {
        return Err(content.error("Expected literal, identifier or block"));
    }
    Ok(value)
}

impl RmlNode {
    
    pub fn generate(&self) -> GenResult {
        use crate::format::{find_related_property_for_binding};
        use quote::{format_ident, quote};
        use uuid::Uuid;
        use rml_core::{ItemTypeEnum, AbstractValue};
        use std::collections::HashMap;
        
        let node_type_str = self._ident.to_string();

        // Only check built-in types if it's not a custom component
        let node_type = match node_type_str.as_str() {
            "Node" => ItemTypeEnum::Node,
            "Rectangle" => ItemTypeEnum::Rectangle,
            "Text" => ItemTypeEnum::Text,
            "MouseArea" => ItemTypeEnum::MouseArea,
            _ => panic!("Unknown node type: {}. Make sure it's either a built-in type (Node, Rectangle, Text, MouseArea) or a custom component imported from components folder.", node_type_str),
        };
        
        // search for the id property and generate a uuid if not found
        let id = self
            .properties
            .iter()
            .find_map(|(k, v)| {
                if k.to_string() == "id" { 
                    Some(v.to_string()) 
                } else { 
                    None 
                }
            })
            .unwrap_or_else(|| format!("generated_id_{}", Uuid::new_v4().simple().to_string()));

        let temp_node = format_ident!("temp_node_{}", id);

        let child_results: Vec<GenResult> = self
            .children
            .iter()
            .map(|child| child.generate())
            .collect();

        let child_code: Vec<proc_macro2::TokenStream> = child_results
            .iter()
            .map(|(_, code, _, _)| code.clone())
            .collect();

        let child_temp_nodes: Vec<proc_macro2::TokenStream> = child_results
            .iter()
            .map(|(id, _, _, _)| {
                let child_temp_var = format_ident!("temp_node_{}", id);
                quote! { #child_temp_var }
            })
            .collect();
        
        let initializer_of_childs: Vec<proc_macro2::TokenStream> = child_results
            .iter()
            .map(|(_, _, _, initializer)| initializer.clone())
            .collect();

        let initializer: Vec<proc_macro2::TokenStream> = self
            .properties
            .iter()
            .map(|(k, v)| {
                let k_string = k.to_string();
                let k_ident = k.to_ident();
                
                // it's a binding
                if !k_string.starts_with("on_") && !k_string.ends_with("_changed") {
                    let value = match v {
                        Value::Block(block) => {
                            // find the property on wich depend the callback, we will need to analyze the block code
                            // and compare the property names in the block with the property names in the engine
                            // get block in string
                            let block_string = format!("{}", quote! { #block });
                            let related_property = find_related_property_for_binding(id.clone(), k_string, block_string);
                            // Generate the binding calls
                            let binding_calls: Vec<proc_macro2::TokenStream> = related_property.iter().map(|(node, prop)| {
                                quote! {
                                    engine.bind_node_property_to_callback(#node, #prop, cb_id);
                                }
                            }).collect();

                            let temp_node_copy = temp_node.clone();
                            quote! {
                                // here we set the property created in the properties part, this code will be executed in the initializer stage
                                let value: AbstractValue = #block .into();
                                let node_name = engine.get_node(#temp_node_copy ).unwrap().id.clone();
                                engine.set_property_of_node(&node_name, stringify!(#k_ident), value);
                                
                                let captured_node = #temp_node;
                                // create a callback to set the property when the related property used in the block changes
                                let cb_id = engine.add_callback(move |engine| {
                                    let value: AbstractValue = #block .into();
                                    let node_name = engine.get_node( captured_node ).unwrap().id.clone();
                                    engine.set_property_of_node(&node_name, stringify!(#k_ident), value);
                                });
                                
                                // bind the property to the callback for each related property
                                #(#binding_calls)*
                            }
                        }
                        _ => { quote! {} }
                    };
                    value
                }
                // it's an initializer
                else  if k_string.starts_with("on_ready") {
                    let value = match v {
                        Value::Block(block) => {
                            quote! {
                                // this code will be executed in the initializer stage
                                #block
                            }
                        }
                        _ => { quote! {} }
                    };
                    value
                }
                else {
                    quote! {}
                }
            }).collect();

        let initializer_code = quote! {
            #(#initializer)*
            #(#initializer_of_childs)*
        };

        let functions: Vec<proc_macro2::TokenStream> = self
            .functions
            .iter()
            .map(|f| {
                let f_name = f.sig.ident.clone();
                let f_inputs = f.sig.inputs.clone();
                let f_output = f.sig.output.clone();
                let f_body = inject_engine_in_block((*f.block).clone(), false);

                let res = match f_output {
                    syn::ReturnType::Default => {
                        quote! { fn #f_name(engine: &mut RmlEngine, #f_inputs) #f_body }
                    }
                    _ => {
                        quote! {
                            fn #f_name(engine: &mut RmlEngine, #f_inputs) #f_output
                            #f_body
                        }
                    }
                };
                res
            })
            .collect();

        let functions_code = quote! {
            #(#functions)*
        };

        // Generate the properties code
        // We need to check if the property is a block with initializer, a callback or a value
        let properties: Vec<proc_macro2::TokenStream> = self
            .properties
            .iter()
            .map(|(k, v)| {
                let k_string = k.to_string();
                let k_ident = k.to_ident();
                
                // Handle signal declarations
                if k.is_signal() {
                    // Signal declaration - create a signal property
                    quote! {
                        let prop_id = engine.add_property(Property::new(AbstractValue::Null));
                        engine.add_property_to_node(#temp_node, stringify!(#k_ident).to_string(), prop_id);
                    }
                } else if k_string.starts_with("on_") && k_string.ends_with("_changed") {
                    // Property change callbacks
                    let observed = k_string.trim_start_matches("on_").trim_end_matches("_changed");
                    if let Value::Block(block) = v {
                        quote! {
                            let cb_id = engine.add_callback( |engine| #block );
                            engine.bind_node_property_to_callback( #id, #observed, cb_id );
                        }
                    } else {
                        quote! {}
                    }
                } else if k_string.starts_with("on_") {
                    let event_name = k_string.trim_start_matches("on_");
                    
                    // Check if this is a custom signal handler
                    let is_custom_signal = self.properties.iter().any(|(prop_key, _)| {
                        prop_key.is_signal() && prop_key.to_string() == event_name
                    });
                    
                    if is_custom_signal {
                        // Custom signal handler
                        if let Value::Block(block) = v {
                            quote! {
                                let cb_id = engine.add_callback( |engine| #block );
                                engine.bind_node_property_to_callback( #id, #event_name, cb_id );
                            }
                        } else {
                            quote! {}
                        }
                    } else {
                        // System event handlers
                        
                        // Check if this is a mouse event and if we're in a MouseArea
                        let is_mouse_event = matches!(event_name, 
                            "mouse_down" | "mouse_up" | "mouse_move" | "mouse_wheel" | 
                            "mouse_enter" | "mouse_leave" | "click"
                        );
                        
                        if is_mouse_event && node_type != ItemTypeEnum::MouseArea {
                            // Mouse events are only allowed on MouseArea nodes
                            panic!("Mouse events can only be used in MouseArea nodes, node is {:?}", node_type);
                        }
                        
                        let event_type = match event_name {
                            "key_down" => quote! { EventType::KeyDown },
                            "key_up" => quote! { EventType::KeyUp },
                            "key_pressed" => quote! { EventType::KeyPressed },
                            "mouse_down" => quote! { EventType::MouseDown },
                            "mouse_up" => quote! { EventType::MouseUp },
                            "mouse_move" => quote! { EventType::MouseMove },
                            "mouse_wheel" => quote! { EventType::MouseWheel },
                            "mouse_enter" => quote! { EventType::MouseEnter },
                            "mouse_leave" => quote! { EventType::MouseLeave },
                            "click" => quote! { EventType::Click },
                            "window_resize" => quote! { EventType::WindowResize },
                            "window_focus" => quote! { EventType::WindowFocus },
                            "window_lost_focus" => quote! { EventType::WindowLostFocus },
                            _ => return quote! {}, // Unknown event type
                        };
                        
                        if let Value::Block(block) = v {
                            quote! {
                                let cb_id = engine.add_callback( |engine| #block );
                                engine.add_event_handler( #event_type, #id, cb_id );
                            }
                        } else {
                            quote! {}
                        }
                    }
                } else {
                    let value = match v {
                        Value::Block(_block) => {
                            quote! {
                                let prop_id = engine.add_property(Property::new( AbstractValue::Null ));
                                engine.add_property_to_node(#temp_node, stringify!(#k_ident).to_string() , prop_id);
                                // we set the property to null for now, we will set it later in the initializer
                            }
                        }
                        _ => {
                            let value = value_to_abstract_value(v);
                            quote! {
                                let prop_id = engine.add_property(Property::new( #value ));
                                engine.add_property_to_node(#temp_node, stringify!(#k_ident).to_string() , prop_id);
                            }   
                        }
                    };
                    value
                }
            }).collect();

        let node_code = quote! {
            let #temp_node = engine.add_node(
                #id.to_string(),
                #node_type,
                HashMap::new(),
            ).unwrap();

            // create computed geometry properties for all nodes
            let computed_x_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "computed_x".to_string(), computed_x_prop);
            let computed_y_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "computed_y".to_string(), computed_y_prop);
            let computed_width_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "computed_width".to_string(), computed_width_prop);
            let computed_height_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "computed_height".to_string(), computed_height_prop);

            #(#properties)*

            #(
                #child_code;
                engine.add_child(#temp_node, #child_temp_nodes);
            )*
        };

        (id, node_code, functions_code, initializer_code)
    }
}