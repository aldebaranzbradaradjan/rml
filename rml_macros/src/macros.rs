
use proc_macro2::Span;
use quote::{format_ident, quote};
use rml_core::prelude::{info, warn, RED};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{Ident, Lit, Token, Expr, ExprPath, Member, LitStr};
use uuid::Uuid;
use rml_core::{AbstractValue, ItemTypeEnum, PropertyName};
use std::collections::HashMap;
use std::fs;
use regex::Regex;
use std::path::Path;

use crate::structs::*;
use crate::format::*;

pub fn load_components_from_path(path: &str) -> Result<Vec<ComponentDefinition>, Box<dyn std::error::Error>> {
    let mut components = Vec::new();
    
    // Get the directory path relative to the current file
    let components_dir = Path::new(path);
    
    if !components_dir.exists() || !components_dir.is_dir() {
        return Err(format!("Component directory '{}' not found", path).into());
    }
    
    // Read all .rml files in the directory
    for entry in fs::read_dir(components_dir)? {
        let entry = entry?;
        let file_path = entry.path();
        
        if file_path.extension().and_then(|s| s.to_str()) == Some("rml") {
            let component_name = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("UnknownComponent")
                .to_string();

            components.push(ComponentDefinition {
                name: component_name,
                path: file_path.as_path().to_str().unwrap().to_string(),
            });
        }
    }
    
    Ok(components)
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
        println!("Parsing block value");
        println!("Content before parsing: {}", content.to_string());

        // Just try to parse the block directly and ignore any $ syntax errors for now
        match content.parse::<syn::Block>() {
            Ok(block) => {
                println!("Parsed block directly");
                value = Value::Block(block);
            }
            Err(_) => {
                println!("Failed to parse block directly, creating dummy block, the content : {}", content.to_string());
                // If parsing fails due to $ syntax, create a dummy block
                value = Value::Ident(Ident::new("test", Span::call_site()));
            }
        }

        println!("Parsed block value");
        println!("Content after parsing: {}", content.to_string());

    } else {
        return Err(content.error("Expected literal, identifier or block"));
    }
    Ok(value)
}


impl RmlParser {
    pub fn empty() -> RmlParser {
        RmlParser { components: HashMap::new(), root_node: RmlNode { _ident: "".to_string(), properties: Vec::new(), children: Vec::new(), functions: Vec::new() } }
    }

    pub fn parse_with_path(input: ParseStream, parent_path: String) -> syn::Result<Self> {
        let mut components = HashMap::new();
        // Parse imports first
        while input.peek(Ident) && input.peek2(LitStr) {
            // Check if this is an import statement by looking ahead
            let fork = input.fork();
            if let Ok(ident) = fork.parse::<Ident>() {
                if ident == "import" {
                    let import: ImportStatement = input.parse()?;
                    
                    // Load components from the import path
                    // Resolve the path: if relative and we have a parent, combine them
                    let resolved_path = if Path::new(&import.path).is_absolute() {
                        // path is absolute
                        import.path.to_string()
                    } else {
                        //get the dir part of the parent path
                        Path::new(&parent_path).parent().unwrap_or(Path::new("")).join(&import.path).to_string_lossy().to_string()
                    };

                    if let Ok(loaded_components) = load_components_from_path(&resolved_path) {
                        for component in loaded_components {
                            let component_name = if let Some(alias) = &import.alias {
                                format!("{}::{}", alias, component.name)
                            } else {
                                component.name.clone()
                            };
                            components.insert(component_name, component);
                        }
                    }
                    continue;
                }
            }
            break;
        }
        
        // Parse the root node
        let root_node: RmlNode = input.parse()?;

        println!("Finished parsing RmlParser");

        Ok(RmlParser {
            components,
            root_node
        })
    }
}


impl Parse for ImportStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse "import" as an identifier
        let import_keyword: Ident = input.parse()?;
        if import_keyword != "import" {
            return Err(syn::Error::new(import_keyword.span(), "Expected 'import'"));
        }
        
        let path_str: LitStr = input.parse()?;
        let path = path_str.value(); //"components".to_string(); 
        
        let alias = if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            let alias_ident: Ident = input.parse()?;
            Some(alias_ident.to_string())
        } else {
            None
        };
        
        Ok(ImportStatement { path, alias })
    }
}

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
                    if let Ok(keyword) = fork.parse::<Ident>() {
                        if keyword == "signal" {
                            // parse signal declaration: signal identifier
                            content.parse::<Ident>()?; // consume "signal" string
                            let signal_name: Ident = content.parse()?;
                            content.parse::<Token![,]>().ok();
                            // add signal as a special property
                            properties.push((PropertyType::Signal, PropertyKey::Signal(signal_name), Value::Ident(Ident::new("signal", Span::call_site()))));
                            continue;
                        }

                        if keyword == "number" || keyword == "bool" || keyword == "string" || keyword == "color" {

                            let property_type = match keyword.to_string().as_str() {
                                "number" => PropertyType::Number,
                                "bool" => PropertyType::Bool,
                                "string" => PropertyType::String,
                                "color" => PropertyType::Color,
                                _ => PropertyType::Unknown,
                            };

                            // parse signal declaration: signal identifier
                            content.parse::<Ident>()?; // consume "signal" string
                            let fork = content.fork();
                            if let Ok(_key) = parse_property_key(&fork) {
                                if fork.peek(Token![:]) {
                                    // assume it's a property
                                    let key = parse_property_key(&content)?;
                                    content.parse::<Token![:]>()?;
                                    println!("Parsing property: {}", key.to_string());
                                    let value = property_parse(&content)?;
                                    content.parse::<Token![,]>().ok();
                                    properties.push((property_type, key, value));
                                    println!("Just parsed property");
                                    continue;
                                }
                            }
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
                        println!("Parsing property: {}", key.to_string());
                        let value = property_parse(&content)?;
                        content.parse::<Token![,]>().ok();
                        properties.push((PropertyType::Unknown, key, value));
                        println!("Just parsed property");
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

        println!("Finished parsing RmlNode: {}", _ident);

        Ok(Self {
            _ident,
            properties,
            children,
            functions,
        })
    }
}


impl RmlNode {

    pub fn generate_with_components_and_counter(&mut self, components: &HashMap<String, ComponentDefinition>, id_counter: &mut u32, properties_mapping: &HashMap<String, AbstractValue>) -> GenResult {
        let node_type_str = self._ident.to_string();

        // Check if this is a custom component
        if let Some(component_def) = components.get(&node_type_str) {
            // For custom components, we expand them by generating the component's node
            // and applying the properties passed to the component
            let cmp = self.generate_custom_component_with_counter(component_def, id_counter, properties_mapping);
            return cmp;
        }

        let node_type = match node_type_str.as_str() {
            "Node" => ItemTypeEnum::Node,
            "Rectangle" => ItemTypeEnum::Rectangle,
            "Text" => ItemTypeEnum::Text,
            "MouseArea" => ItemTypeEnum::MouseArea,
            _ => panic!("Unknown node type: {}", node_type_str),
        };
        
        // search for the id property and generate a uuid if not found
        let id = self
            .properties
            .iter()
            .find_map(|(t, k, v)| {
                if k.to_string() == "id" { 
                    Some(v.to_string()) 
                } else { 
                    None 
                }
            })
            .unwrap_or_else(|| {
                let n_id = format!("generated_id_{}", id_counter);
                *id_counter += 1;
                n_id
            });

        println!("real new id  {} and counter : {}", id, id_counter);

        let temp_node = format_ident!("temp_node_{}", id);

        let child_results: Vec<GenResult> = self
            .children
            //.iter()
            .iter_mut()
            .map(|child| child.generate_with_components_and_counter(components, id_counter, properties_mapping))
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
            .map(|(t, k, v)| {
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
            .map(|(t, k, v)| {
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
                    let is_custom_signal = self.properties.iter().any(|(_, prop_key, _)| {
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
                            panic!("Mouse events can only be used in MouseArea nodes");
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

            // create geometry properties for all nodes
            let x_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "x".to_string(), x_prop);
            let y_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "y".to_string(), y_prop);
            let width_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "width".to_string(), width_prop);
            let height_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "height".to_string(), height_prop);

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

    fn generate_custom_component_with_counter(&self, component_def: &ComponentDefinition, id_counter: &mut u32, properties_mapping: &HashMap<String, AbstractValue>) -> GenResult {
        // Read and parse the component file
        let file_content = fs::read_to_string(&component_def.path).unwrap();

        // we need to perform the id diversification in advance
        // to be able to find it in the properties_mapping

        // find id: value in file_content with a regex (without quotes)
        let re = Regex::new(r#"id:\s*(\w+)"#).unwrap();
        let file = file_content.clone();
        let original_id = re.captures(&file)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .unwrap_or("");


        let n_id = format!("generated_id_{}", id_counter);
        *id_counter += 1;

        // maybe there is id property in self.properties, so we need to use that instead
        let n_id = self.properties.iter().find_map(|(t, k, v)| {
            if k.to_string() == "id" { 
                Some(v.to_string()) 
            } else { 
                None 
            }
        }).unwrap_or_else(|| n_id);


        let file_content = if !original_id.is_empty() {
            file_content.replace(original_id, &n_id)
        } else {
            file_content
        };

        println!("Custom component original id: {}, new id: {}", original_id, n_id);

        // display properties_mapping and check if we have the n_id there
        //println!("Custom component properties mapping: {:#?}", properties_mapping);
        println!("Custom component properties mapping contains n_id: {}", properties_mapping.contains_key(&n_id));

        let file_content = transform_dollar_syntax(&file_content, properties_mapping);
        let tokens: proc_macro::TokenStream = file_content.parse().unwrap();

        let res= syn::parse::Parser::parse(|input: ParseStream| {
          RmlParser::parse_with_path(input, component_def.path.clone())
        }, tokens.clone()).unwrap();

        let (mut component_node, components) = (res.root_node, res.components);

        // let original_id: String = match component_node.properties.iter().find(|(k, _)| k.to_string() == "id".to_string()) {
        //     Some(id) => id.1.to_string(),
        //     None => String::from(""),
        // };

        // remove id property if exist
        //component_node.properties.retain(|(k, _)| k.to_string() != "id".to_string());

        // Override the component's properties with the ones passed to this instance
        for (prop_type, prop_key, prop_value) in &self.properties {
            // Find if this property already exists in the component
            if let Some(existing_prop) = component_node.properties.iter_mut().find(|(_, k, _)| k.to_string() == prop_key.to_string()) {
                existing_prop.2 = prop_value.clone();
            } else {
                // Add new property
                component_node.properties.push((prop_type.clone(), prop_key.clone(), prop_value.clone()));
            }
        }
        
        // Add children from this instance to the component
        component_node.children.extend(self.children.clone());
        
        // Generate the component with the applied properties
        let component_gen_res = component_node.generate_with_components_and_counter(&components, id_counter, properties_mapping);
        let new_id = component_gen_res.0.clone();

        // replace the original id present in the component (in callbacks) with the new id
        let mut component_code = component_gen_res.1;
        let mut functions_code = component_gen_res.2;
        let mut initializer_code = component_gen_res.3;
        
        // Only do replacement if we have an original_id to replace
        if !original_id.is_empty() {
            // Replace in component code
            let component_str = component_code.to_string();
            let new_component_str = component_str.replace(&original_id, &new_id);
            //let new_component_str = transform_dollar_syntax(&new_component_str);
            component_code = new_component_str.parse().unwrap_or(component_code);
            
            // Replace in functions code
            let functions_str = functions_code.to_string();
            let new_functions_str = functions_str.replace(&original_id, &new_id);
            //let new_functions_str = transform_dollar_syntax(&new_functions_str);
            functions_code = new_functions_str.parse().unwrap_or(functions_code);
            
            // Replace in initializer code
            let initializer_str = initializer_code.to_string();
            let new_initializer_str = initializer_str.replace(&original_id, &new_id);
            //let new_initializer_str = transform_dollar_syntax(&new_initializer_str);
            initializer_code = new_initializer_str.parse().unwrap_or(initializer_code);
        }

        (new_id, component_code, functions_code, initializer_code)
    }

    pub fn pre_generate_with_components_and_counter(&mut self, components: &HashMap<String, ComponentDefinition>, id_counter: &mut u32) -> HashMap<String, AbstractValue> {
        println!("pre_generate_with_components");
        let node_type_str = self._ident.to_string();

        // Check if this is a custom component
        if let Some(component_def) = components.get(&node_type_str) {
            // For custom components, we expand them by generating the component's node
            // and applying the properties passed to the component
            let cmp = self.pre_generate_custom_component_with_counter(component_def, id_counter);
            return cmp;
        }
        
        // search for the id property and generate a uuid if not found
        let id: String = self
            .properties
            .iter()
            .find_map(|(t, k, v)| {
                if k.to_string() == "id" { 
                    Some(v.to_string()) 
                } else { 
                    None 
                }
            })
            .unwrap_or_else(|| {
                let n_id = format!("generated_id_{}", id_counter);
                *id_counter += 1;
                n_id
            });

        println!("new id  {} and counter : {}", id, id_counter);

        // for each child, we need to merge the hashmap into a single one
        let mut merged_child_results: HashMap<String, AbstractValue> = HashMap::new();
        for i in 0..self.children.len() {
            merged_child_results.extend(self.children[i].pre_generate_with_components_and_counter(components, id_counter));
        }

        // Generate the properties code
        // We need to check if the property is a block with initializer, a callback or a value

        // by typing the property system we can avoid some issues later
        // we must impl sometthing like : 
        /*
        pub enum PropertyKey {
            Simple(
                type: Type,
                base: Ident
            ),
            Composed {
                type: AbstractValue,
                base: Ident, 
                field: Ident 
            },
            Signal(Ident),
        }

        and support syntaxe like this in parser :
         let mut engine = rml!(
        import "components" as Components

        Node {
            id: root
            number width: 500.0
            number height: 500.0

            Components::Button {
                id: counter_btn
                anchors: center
                number counter: 0
                string text: { format!("Counter: {}", $.counter_btn.counter) }
                on_click: { $.counter_btn.counter += 1.0; }
                number val: 10.0
            }
        }
    );

    with special case id with type id and anchors
        */

        let mut properties: HashMap<String, AbstractValue> = self
            .properties
            .iter()
            .map(|(t, k, v)| {
                let k_string = k.to_string();
                
                // Handle signal declarations
                if k.is_signal() {
                    (format!("{}.{}", id, k.to_string()), AbstractValue::Null)
                } else if k_string.starts_with("on_") && k_string.ends_with("_changed") {
                    (format!("{}.{}", id, k.to_string()), AbstractValue::Null)
                } else if k_string.starts_with("on_") {
                    (format!("{}.{}", id, k.to_string()), AbstractValue::Null)
                } else {
                    // let value = match v {

                    //     // here use type to know what to put istead of null

                    //     Value::Block(_block) => {
                    //         let value_type= match t {
                    //             PropertyType::Number => AbstractValue::Number(0.0),
                    //             PropertyType::Bool => AbstractValue::Bool(false),
                    //             PropertyType::String => AbstractValue::String("".to_string()),
                    //             PropertyType::Color => AbstractValue::Color(RED),
                    //             _ => AbstractValue::Null,
                    //         };

                    //         (format!("{}.{}", id, k.to_string()), value_type)
                    //     }
                    //     _ => {
                    //         let value = value_to_abstract_value(v);
                    //         (format!("{}.{}", id, k.to_string()), value)  
                    //     }
                    // };

                    let value_type= match t {
                        PropertyType::Number => AbstractValue::Number(0.0),
                        PropertyType::Bool => AbstractValue::Bool(false),
                        PropertyType::String => AbstractValue::String("".to_string()),
                        PropertyType::Color => AbstractValue::Color(RED),
                        _ => AbstractValue::Null,
                    };

                    (format!("{}.{}", id, k.to_string()), value_type)
                    //value
                }
            }).collect();

        //println!("Props : {:#?}", properties);

        properties.extend(merged_child_results);

        if properties.get(&format!("{}.x", id)).is_none() {
            properties.insert(format!("{}.{}", id, "x"), AbstractValue::Number(0.0));
        }
        if properties.get(&format!("{}.y", id)).is_none() {
            properties.insert(format!("{}.{}", id, "y"), AbstractValue::Number(0.0));
        }
        if properties.get(&format!("{}.width", id)).is_none() {
            properties.insert(format!("{}.{}", id, "width"), AbstractValue::Number(0.0));
        }
        if properties.get(&format!("{}.height", id)).is_none() {
            properties.insert(format!("{}.{}", id, "height"), AbstractValue::Number(0.0));
        }
        if properties.get(&format!("{}.computed_x", id)).is_none() {
            properties.insert(format!("{}.{}", id, "computed_x"), AbstractValue::Number(0.0));
        }
        if properties.get(&format!("{}.computed_y", id)).is_none() {
            properties.insert(format!("{}.{}", id, "computed_y"), AbstractValue::Number(0.0));
        }
        if properties.get(&format!("{}.computed_width", id)).is_none() {
            properties.insert(format!("{}.{}", id, "computed_width"), AbstractValue::Number(0.0));
        }
        if properties.get(&format!("{}.computed_height", id)).is_none() {
            properties.insert(format!("{}.{}", id, "computed_height"), AbstractValue::Number(0.0));
        }

        properties
    }
    
    fn pre_generate_custom_component_with_counter(&self, component_def: &ComponentDefinition, id_counter: &mut u32) -> HashMap<String, AbstractValue> {
        println!("pre_generate_custom_component");
        // Read and parse the component file
        let file_content = fs::read_to_string(&component_def.path).unwrap();
        let tokens: proc_macro::TokenStream = file_content.parse().unwrap();

        let mut res = RmlParser::empty();
        match syn::parse::Parser::parse(|input: ParseStream| {
            res = RmlParser::parse_with_path(input, component_def.path.clone()).unwrap();
            Ok(RmlParser::empty())
        }, tokens.clone()) {
            Ok(r) => r,
            Err(_) => {
                RmlParser::empty()
            }
        };

        let (mut component_node, components) = (res.root_node, res.components);

        // remove id property if exist
        component_node.properties.retain(|(_, k, _)| k.to_string() != "id".to_string());

        // Override the component's properties with the ones passed to this instance
        for (prop_type, prop_key, prop_value) in &self.properties {
            // Find if this property already exists in the component
            if let Some(existing_prop) = component_node.properties.iter_mut().find(|(_, k, _)| k.to_string() == prop_key.to_string()) {
                existing_prop.2 = prop_value.clone();
                // warning if property type is set in this instance
                // if *prop_type != PropertyType::Unknown {
                //     warn!("Property type defined in component cannot be overridden: {:#?} in {}", prop_key, component_def.name);
                // }

                if !matches!(prop_type, PropertyType::Unknown) {
                    warn!("Property type defined in component cannot be overridden: {:#?} in {}", prop_key, component_def.name);
                }

            } else {
                // Add new property
                component_node.properties.push((prop_type.clone(), prop_key.clone(), prop_value.clone()));
            }
        }
        
        // Add children from this instance to the component
        component_node.children.extend(self.children.clone());
        
        // Generate the component with the applied properties
        component_node.pre_generate_with_components_and_counter(&components, id_counter)
    }
}