
use std::process::{Command, Stdio};

pub fn format_code_for_binding_extraction(code: &str) -> String {
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

pub fn format_code(code: &str) -> String {
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

pub fn transform_dollar_syntax(code: &str) -> String {
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

pub fn inject_engine_text_based(
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


pub fn find_related_property_for_binding(id: String, property: String, block_string: String) -> Vec<(String, String)> {
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
