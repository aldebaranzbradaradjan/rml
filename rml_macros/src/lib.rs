use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};

use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{parse_macro_input, Ident, Lit, Token, Expr, ExprPath, Member, LitStr};

use uuid::Uuid;

use rml_core::{AbstractValue, ItemTypeEnum};

use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// struct to represent a property key
// It can be a simple identifier or a composed one (base.field)
#[derive(Debug, Clone)]
enum PropertyKey {
    Simple(Ident),
    Composed { 
        base: Ident, 
        field: Ident 
    },
    Signal(Ident),
}

// struct to represent an import statement
#[derive(Debug, Clone)]
struct ImportStatement {
    path: String,
    alias: Option<String>,
}

// Component definition parsed from a .rml file
#[derive(Clone)]
struct ComponentDefinition {
    name: String,
    node: RmlNode,
}

impl PropertyKey {
    fn to_string(&self) -> String {
        match self {
            PropertyKey::Simple(ident) => ident.to_string(),
            PropertyKey::Composed { base, field } => format!("{}_{}", base, field),
            PropertyKey::Signal(ident) => ident.to_string(),
        }
    }
    
    fn to_ident(&self) -> Ident {
        format_ident!("{}", self.to_string())
    }
    
    fn is_signal(&self) -> bool {
        matches!(self, PropertyKey::Signal(_))
    }
}

fn format_code_for_binding_extraction(code: &str) -> String {
    // remove line jumps;
    let mut code = code.replace("\n", "").replace("\r", "");
    // add line jump before get macro calls
    let macros = [ "get_value!", "get_number!", "get_string!",
    "get_bool!", "get_color!", "get_computed_x!", "get_computed_y!",
    "get_computed_width!", "get_computed_height!",
    "get_number_property_of_node", "get_string_property_of_node", "get_bool_property_of_node",
    "get_color_property_of_node", "get_property_of_node" ];
    for macro_name in macros {
        code = code.replace(macro_name, &format!("\n{}", macro_name));
    }
    code.to_string()
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

fn transform_dollar_syntax(code: &str) -> String {
    use regex::Regex;
    
    // Only transform if there are actually $ expressions
    if !code.contains("$.") {
        return code.to_string();
    }
    
    let mut result = code.to_string();
    
    // Handle compound assignments first: $.node.prop += value;
    let compound_assign_pattern = Regex::new(r"\$\.([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)\s*([+\-*/])=\s*([^;]+)\s*;").unwrap();
    result = compound_assign_pattern.replace_all(&result, |caps: &regex::Captures| {
        let node_id = &caps[1];
        let property = &caps[2];
        let operator = &caps[3];
        let value = &caps[4].trim();
        
        format!("set_number!(engine, {}, {}, get_number!(engine, {}, {}) {} {});", 
                node_id, property, node_id, property, operator, value)
    }).to_string();
    
    // Handle simple assignments: $.node.prop = value;
    let assign_pattern = Regex::new(r"\$\.([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*([^;]+)\s*;").unwrap();
    result = assign_pattern.replace_all(&result, |caps: &regex::Captures| {
        let node_id = &caps[1];
        let property = &caps[2];
        let value = &caps[3].trim();
        
        // Try to determine the type based on the value
        if value.starts_with('"') || value.contains(".to_string()") || value.contains("String::") {
            format!("set_string!(engine, {}, {}, {});", node_id, property, value)
        } else if *value == "true" || *value == "false" {
            format!("set_bool!(engine, {}, {}, {});", node_id, property, value)
        } else {
            format!("set_number!(engine, {}, {}, {});", node_id, property, value)
        }
    }).to_string();
    
    // Handle typed read operations first: $.node.prop:type
    let typed_pattern = Regex::new(r"\$\.([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z0-9_]+)\b").unwrap();
    result = typed_pattern.replace_all(&result, |caps: &regex::Captures| {
        let node_id = &caps[1];
        let property = &caps[2];
        let type_hint = &caps[3];
        
        match type_hint {
            "f32" | "number" => format!("get_number!(engine, {}, {})", node_id, property),
            "string" | "str" => format!("get_string!(engine, {}, {})", node_id, property),
            "bool" => format!("get_bool!(engine, {}, {})", node_id, property),
            "color" => format!("get_color!(engine, {}, {})", node_id, property),
            _ => format!("get_value!(engine, {}, {})", node_id, property),
        }
    }).to_string();
    
    // Handle regular read operations: $.node.prop (in expressions)
    // Be more careful to only match standalone expressions, not inside strings
    let dollar_pattern = Regex::new(r"\$\.([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();
    result = dollar_pattern.replace_all(&result, |caps: &regex::Captures| {
        let node_id = &caps[1];
        let property = &caps[2];
        
        // Use the new get_value! macro that returns the AbstractValue
        // The context will determine how it's used (to_string(), to_number(), etc.)
        format!("get_value!(engine, {}, {})", node_id, property)
    }).to_string();
    
    result
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

fn load_components_from_path(path: &str) -> Result<Vec<ComponentDefinition>, Box<dyn std::error::Error>> {
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
            
            // Read and parse the component file
            let file_content = fs::read_to_string(&file_path)?;
            let file_content = transform_dollar_syntax(&file_content);
            
            // Parse the RML content as a component
            if let Ok(component_node) = parse_component_content(&file_content) {
                components.push(ComponentDefinition {
                    name: component_name,
                    node: component_node,
                });
            }
        }
    }
    
    Ok(components)
}

fn parse_component_content(content: &str) -> Result<RmlNode, Box<dyn std::error::Error>> {
    // Parse the content as a TokenStream and then as an RmlNode
    let tokens: proc_macro2::TokenStream = content.parse()?;
    let parsed = syn::parse2::<RmlNode>(tokens)?;
    Ok(parsed)
}


#[proc_macro]
pub fn rml(input: TokenStream) -> TokenStream {
    // First, transform the input to replace $ syntax before parsing
    let input_string = input.to_string();
    let transformed_string = transform_dollar_syntax(&input_string);
    
    // Parse the transformed input
    let transformed_input: TokenStream = match transformed_string.parse() {
        Ok(tokens) => { println!("Transformed input"); tokens },
        Err(_) => { println!("original input"); input }, // Fallback to original if transformation fails
    };
    
    // Try to parse as RmlParser first (with imports), fallback to RmlNode for backward compatibility
    let (parsed_node, components) = if let Ok(rml_parser) = syn::parse::<RmlParser>(transformed_input.clone()) {
        (rml_parser.root_node, rml_parser.components)
    } else {
        let parsed = parse_macro_input!(transformed_input as RmlNode);
        (parsed, HashMap::new())
    };

    println!("Components: {:#?}", components.keys());
    
    let generated = parsed_node.generate_with_components(&components);
    let generated_node = generated.1;
    let generated_functions = generated.2;
    let generated_initializer = generated.3;

    let functions_name: Vec<String> = parsed_node
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

#[derive(Clone)]
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

fn find_related_property_for_binding(id: String, property: String, block_string: String) -> Vec<(String, String)> {
    // ex: k_string = "x", block_string =
    // "{
    // let outer_rect_width = get_number!(engine, outer_rect, width);
    // let inner_rect_width = get_number!(engine, inner_rect, width);
    // let inner_rect_width = engine.get_number_property_of_node(inner_rect, "width", 0.0);
    // outer_rect_width / 2.0 - inner_rect_width / 2.0
    // }"
    // will return [(outer_rect, width), (inner_rect, width)]
    let block_string = format_code_for_binding_extraction(block_string.as_str());
    let mut related_properties = Vec::new();
    
    // if in block we find get_number!, get_string!, get_bool!, get_color!
    // get_computed_x!, get_computed_y!, get_computed_width!, get_computed_height!
    // get_number_property_of_node, get_string_property_of_node, get_bool_property_of_node, get_color_property_of_node
    // or get_property_of_node
    // we will add it to related_properties
    for line in block_string.lines() {
        let trimmed_line = line.trim();

        println!("Line: {}", trimmed_line);
        
        if trimmed_line.contains("get_value!") ||trimmed_line.contains("get_number!") || trimmed_line.contains("get_string!") || 
           trimmed_line.contains("get_bool!") || trimmed_line.contains("get_color!") ||
           trimmed_line.contains("get_computed_x!") || trimmed_line.contains("get_computed_y!") || 
           trimmed_line.contains("get_computed_width!") || trimmed_line.contains("get_computed_height!") {
            
            // Parse macro calls like get_number!(engine, node_name, property_name)
            if let Some(start) = trimmed_line.find('(') {
                if let Some(end) = trimmed_line.find(')') {
                    let params = &trimmed_line[start + 1..end];
                    let parts: Vec<&str> = params.split(',').map(|s| s.trim()).collect();
                    
                    if parts.len() >= 3 {
                        let node_name = parts[1].trim();
                        let property_name = parts[2].trim().trim_matches('"');
                        if node_name == id && property_name == property {
                            continue;
                        }
                        related_properties.push((node_name.to_string(), property_name.to_string()));
                    }
                }
            }
        }
        else if trimmed_line.contains("get_number_property_of_node") || 
                trimmed_line.contains("get_string_property_of_node") ||
                trimmed_line.contains("get_bool_property_of_node") || 
                trimmed_line.contains("get_color_property_of_node") || 
                trimmed_line.contains("get_property_of_node") {
            
            // Parse method calls like engine.get_number_property_of_node(node_name, "property_name", default)
            if let Some(start) = trimmed_line.find('(') {
                if let Some(end) = trimmed_line.rfind(')') {
                    let params = &trimmed_line[start + 1..end];
                    let parts: Vec<&str> = params.split(',').map(|s| s.trim()).collect();
                    
                    if parts.len() >= 2 {
                        let node_name = parts[0].trim();
                        let property_name = parts[1].trim().trim_matches('"');
                        if node_name == id && property_name == property {
                            continue;
                        }
                        related_properties.push((node_name.to_string(), property_name.to_string()));
                    }
                }
            }
        }
    }
    
    // Remove duplicates while preserving order
    let mut seen = std::collections::HashSet::new();
    related_properties.retain(|item| seen.insert(item.clone()));
    
    related_properties
}

/// Struct to parse a Node
#[derive(Clone)]
struct RmlNode {
    _ident: Ident,
    properties: Vec<(PropertyKey, Value)>,
    children: Vec<RmlNode>,
    functions: Vec<syn::ItemFn>,
}

/// Main RML parser that includes imports
struct RmlParser {
    components: HashMap<String, ComponentDefinition>,
    root_node: RmlNode,
}

impl Parse for ImportStatement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse "import" as an identifier
        println!("Parse import statement");
        let import_keyword: Ident = input.parse()?;
        if import_keyword != "import" {
            return Err(syn::Error::new(import_keyword.span(), "Expected 'import'"));
        }
        
        println!("Parse import path");
        let path_str: LitStr = input.parse()?;
        let path = path_str.value();
        println!("Import path: {}", path);
        
        let alias = if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            let alias_ident: Ident = input.parse()?;
            Some(alias_ident.to_string())
        } else {
            None
        };

        println!("Alias: {:#?}", alias);
        
        Ok(ImportStatement { path, alias })
    }
}

impl Parse for RmlParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut components = HashMap::new();
        
        // Parse imports first
        while input.peek(Ident) && input.peek2(LitStr) {
            // Check if this is an import statement by looking ahead
            let fork = input.fork();
            if let Ok(ident) = fork.parse::<Ident>() {
                if ident == "import" {
                    let import: ImportStatement = input.parse()?;
                    
                    // Load components from the import path
                    if let Ok(loaded_components) = load_components_from_path(&import.path) {
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
        
        Ok(RmlParser {
            components,
            root_node,
        })
    }
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

type GenResult = (String, proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream);

impl RmlNode {
    // fn generate(&self) -> GenResult {
    //     self.generate_with_components(&HashMap::new())
    // }
    
    fn generate_with_components(&self, components: &HashMap<String, ComponentDefinition>) -> GenResult {
        let node_type_str = self._ident.to_string();

        // Check if this is a custom component
        if let Some(component_def) = components.get(&node_type_str) {
            // For custom components, we expand them by generating the component's node
            // and applying the properties passed to the component
            println!("Generating custom component: {}", node_type_str);
            let cmp = self.generate_custom_component(component_def, components);
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
    
    fn generate_custom_component(&self, component_def: &ComponentDefinition, components: &HashMap<String, ComponentDefinition>) -> GenResult {
        // Clone the component's node and apply the properties from this instance
        let mut component_node = component_def.node.clone();
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
        let component_gen_res = component_node.generate_with_components(components);
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