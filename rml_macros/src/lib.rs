use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};

use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{parse_macro_input, Ident, Lit, Token, Expr, File, Item, ExprCall, ExprPath, Member};

use uuid::Uuid;

use rml_core::{AbstractValue, ItemTypeEnum, Property, RmlEngine, EventType};

use std::process::{Command, Stdio};

// struct to represent a property key
// It can be a simple identifier or a composed one (base.field)
#[derive(Debug, Clone)]
enum PropertyKey {
    Simple(Ident),
    Composed { 
        base: Ident, 
        field: Ident 
    },
}

impl PropertyKey {
    fn to_string(&self) -> String {
        match self {
            PropertyKey::Simple(ident) => ident.to_string(),
            PropertyKey::Composed { base, field } => format!("{}_{}", base, field),
        }
    }
    
    fn to_ident(&self) -> Ident {
        format_ident!("{}", self.to_string())
    }
}

fn format_code(code: &str) -> String {
    let mut rustfmt = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run rustfmt");

    {
        use std::io::Write;
        let stdin = rustfmt.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(code.as_bytes()).expect("Failed to write code");
    }

    let output = rustfmt.wait_with_output().expect("Failed to read output");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn inject_engine_text_based(
    input: &str,
    engine_str: &str,
    definition: bool,
    mutable: bool,
    functions: &Vec<String>,
) -> String {
    let engine_str = if definition {
        format!("&mut {engine_str}")
    } else {
        if mutable {
            format!("&mut {engine_str}")
        } else {
            format!("{engine_str}")
        }
    };

    let mut output = String::new();

    for line in input.lines() {
        let mut modified_line = line.to_string();
        for func in functions {
            let pattern = format!("fn {func}(");

            if line.contains(&pattern) {
                // it's a function definition, we dont need to modify it, its already done in inject_engine_in_block
            } else {
                let pattern = format!("{func}(");
                let replacement = format!("{func}({engine_str}");
                modified_line = modified_line.replace(&pattern, &replacement);
            }
        }

        output.push_str(&modified_line);
        output.push('\n');
    }

    output
}

#[proc_macro]
pub fn rml(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as RmlNode);
    let generated = parsed.generate();
    let generated_node = generated.1;
    let generated_functions = generated.2;
    let generated_initializer = generated.3;

    let mut functions_name: Vec<String> = parsed
        .functions
        .iter()
        .map(|f| f.sig.ident.to_string())
        .collect();

    let generated_initializer = generated_initializer.to_string();
    let generated_initializer = inject_engine_text_based(&generated_initializer, "engine", false, true, &functions_name);
    let generated_initializer = generated_initializer.parse::<proc_macro2::TokenStream>().unwrap();

    let generated_functions = generated_functions.to_string();
    let generated_functions = format_code(&generated_functions);
    let generated_functions = inject_engine_text_based(&generated_functions, "engine", true, true, &functions_name);
    let generated_functions = generated_functions.parse::<proc_macro2::TokenStream>().unwrap();

    let generated_node = generated_node.to_string();
    let generated_node = inject_engine_text_based(&generated_node, "engine", false, false, &functions_name);
    let generated_node = generated_node.parse::<proc_macro2::TokenStream>().unwrap();

    let result = quote! {
        {
            let mut engine = RmlEngine::new();
            #generated_node;
            #generated_functions
            #generated_initializer
            engine
        }
    };
    TokenStream::from(result)
}

enum Value {
    Lit(Lit),
    Ident(Ident),
    Block(syn::Block),
}

fn lit_to_string(literal: &Lit) -> Option<String> {
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

fn value_to_abstract_value(value: &Value) -> AbstractValue {
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

fn inject_engine_in_block(mut block: syn::Block, initializer: bool) -> syn::Block {
    use syn::{Expr, ExprCall, ExprPath, Stmt, Token};

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
fn parse_property_key(input: ParseStream) -> syn::Result<PropertyKey> {
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

fn property_parse(content: &ParseBuffer) -> Result<Value, syn::Error> {
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

/// Struct to parse a Node
struct RmlNode {
    _ident: Ident,
    properties: Vec<(PropertyKey, Value)>,
    children: Vec<RmlNode>,
    functions: Vec<syn::ItemFn>,
}

impl Parse for RmlNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ident: Ident = input.parse()?;
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
                // will try to parse the property first
                let fork = content.fork();
                
                // try to parse a property key
                if let Ok(key) = parse_property_key(&fork) {
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

type GenResult = (String, proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream);

impl RmlNode {
    fn generate(&self) -> GenResult {
        let node_type = self._ident.to_string();

        let node_type = match node_type.as_str() {
            "Node" => ItemTypeEnum::Node,
            "Rectangle" => ItemTypeEnum::Rectangle,
            "Text" => ItemTypeEnum::Text,
            "MouseArea" => ItemTypeEnum::MouseArea,
            _ => panic!("Unknown node type: {}", node_type),
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
            .unwrap_or_else(|| Uuid::new_v4().simple().to_string());

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
                let k_ident = k.to_ident(); // Nouveau helper
                
                if !k_string.starts_with("on_") && !k_string.ends_with("_changed") {
                    let value = match v {
                        Value::Block(block) => {
                            quote! {
                                // here we set the property created in the properties part, this code will be executed in the initializer stage
                                let value: AbstractValue = #block .into();
                                let node_name = engine.get_node(#temp_node).unwrap().id.clone();
                                engine.set_property_of_node(&node_name, stringify!(#k_ident), value);
                            }
                        }
                        _ => { quote! {} }
                    };
                    value
                } else {
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
                
                if k_string.starts_with("on_") && k_string.ends_with("_changed") {
                    // Property change callbacks (existing functionality)
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
                    // System event handlers
                    let event_name = k_string.trim_start_matches("on_");
                    
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
                } else {
                    let value = match v {
                        Value::Block(block) => {
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
            let computed_abs_x_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "computed_absolute_x".to_string(), computed_abs_x_prop);
            let computed_abs_y_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "computed_absolute_y".to_string(), computed_abs_y_prop);
            let computed_width_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "computed_width".to_string(), computed_width_prop);
            let computed_height_prop = engine.add_property(Property::new(AbstractValue::Number(0.0)));
            engine.add_property_to_node(#temp_node, "computed_height".to_string(), computed_height_prop);

            // create a visible property for all nodes
            let visible_prop = engine.add_property(Property::new(AbstractValue::Bool(true)));
            engine.add_property_to_node(#temp_node, "visible".to_string(), visible_prop);

            #(#properties)*

            #(
                #child_code;
                engine.add_child(#temp_node, #child_temp_nodes);
            )*
        };

        (id, node_code, functions_code, initializer_code)
    }
}