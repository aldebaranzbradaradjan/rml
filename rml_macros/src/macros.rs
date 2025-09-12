
use proc_macro2::Span;
use quote::{format_ident, quote};
use rml_core::prelude::info;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{Ident, Lit, Token, Expr, ExprPath, Member, LitStr};
use uuid::Uuid;
use rml_core::{ItemTypeEnum};
use std::collections::HashMap;
use std::fs;
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
        let block: syn::Block = content.parse()?;
        value = Value::Block(block);
    } else {
        return Err(content.error("Expected literal, identifier or block"));
    }
    Ok(value)
}


impl RmlParser {
    pub fn parse_with_path(input: ParseStream, parent_path: String) -> syn::Result<Self> {
        let mut components = HashMap::new();
        info!("RmlParser::parse");
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
                        info!("Parent path: {}", &parent_path);
                        Path::new(&parent_path).parent().unwrap_or(Path::new("")).join(&import.path).to_string_lossy().to_string()
                    };

                    info!("Resolved path: {}", &resolved_path);

                    if let Ok(loaded_components) = load_components_from_path(&resolved_path) {
                        for component in loaded_components {
                            let component_name = if let Some(alias) = &import.alias {
                                format!("{}::{}", alias, component.name)
                            } else {
                                component.name.clone()
                            };
                            
                            println!("Component name: {} {} {}", component_name, component.name, component.path);
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
        
        Ok(RmlParser {
            components,
            root_node,
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

        println!("Alias: {:#?}, path: {:#?}", alias, path);
        
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

        println!("Node name: {}", _ident);

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


impl RmlNode {
    
    pub fn generate_with_components(&self, components: &HashMap<String, ComponentDefinition>) -> GenResult {
        let node_type_str = self._ident.to_string();

        // Check if this is a custom component
        if let Some(component_def) = components.get(&node_type_str) {
            // For custom components, we expand them by generating the component's node
            // and applying the properties passed to the component
            println!("Generating custom component: {}", node_type_str);
            let cmp = self.generate_custom_component(component_def);
            //println!("Generated custom component: {} {}", cmp.0, cmp.1);
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
            .map(|child| child.generate_with_components(components))
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
    
    fn generate_custom_component(&self, component_def: &ComponentDefinition) -> GenResult {
        // Read and parse the component file
        info!("Reading component file: {}", component_def.path);

        let file_content = fs::read_to_string(&component_def.path).unwrap();
        let file_content = transform_dollar_syntax(&file_content);

        let tokens: proc_macro::TokenStream = file_content.parse().unwrap();

        
        //let res = syn::parse::<RmlParser>(tokens.clone()).unwrap();
        let res= syn::parse::Parser::parse(|input: ParseStream| {
          RmlParser::parse_with_path(input, component_def.path.clone())
        }, tokens.clone()).unwrap();

        let (mut component_node, components) = (res.root_node, res.components);


        let original_id: String = match component_node.properties.iter().find(|(k, _)| k.to_string() == "id".to_string()) {
            Some(id) => id.1.to_string(),
            None => String::from(""),
        };

        // remove id property if exist
        component_node.properties.retain(|(k, _)| k.to_string() != "id".to_string());

        // Override the component's properties with the ones passed to this instance
        for (prop_key, prop_value) in &self.properties {
            // Find if this property already exists in the component
            if let Some(existing_prop) = component_node.properties.iter_mut().find(|(k, _)| k.to_string() == prop_key.to_string()) {
                existing_prop.1 = prop_value.clone();
            } else {
                // Add new property
                component_node.properties.push((prop_key.clone(), prop_value.clone()));
            }
        }
        
        // Add children from this instance to the component
        component_node.children.extend(self.children.clone());
        
        // Generate the component with the applied properties
        let component_gen_res = component_node.generate_with_components(&components);
        let new_id = component_gen_res.0.clone();

        // replace the original id present in the component (in callbacks) with the new id
        //println!("original_id: {}, new_id: {}", original_id, new_id);
        
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
}