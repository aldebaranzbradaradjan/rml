use std::{
    collections::HashMap,
    process::{Command, Stdio},
};

use regex::Regex;

use rml_core::{AbstractValue};

pub fn collect_children<'a>(
    node_hierarchy: &'a Vec<(String, Vec<String>)>,
    id: &'a str,
    out: &mut Vec<&'a str>,
) {
    // find the node and its children
    let children = node_hierarchy
        .iter()
        .find(|(curr_id, _)| curr_id == id)
        .map(|(_, v)| v)
        .unwrap();

    // iterate over the childrens
    for child in children {
        out.push(child.as_str());
        collect_children(node_hierarchy, child, out); // recurse
    }
}

pub fn format_code_for_binding_extraction(code: &str) -> String {
    // remove line jumps;
    let mut code = code.replace("\n", "").replace("\r", "");
    // add line jump before get macro calls
    let macros = [
        "get_value!",
        "get_number!",
        "get_string!",
        "get_bool!",
        "get_color!",
        "get_computed_x!",
        "get_computed_y!",
        "get_computed_width!",
        "get_computed_height!",
        "get_number_property_of_node",
        "get_string_property_of_node",
        "get_bool_property_of_node",
        "get_color_property_of_node",
        "get_property_of_node",
    ];
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
        stdin
            .write_all(code.as_bytes())
            .expect("Failed to write code");
    }

    let output = rustfmt.wait_with_output().expect("Failed to read output");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

// pub fn extract_dollar_children_syntax(block_string: String) -> Vec<(String, u32)> {
//     // dollar_ nodeidstr _children_ index _ property
//     let mut children = Vec::new();
//     let children_pattern = Regex::new(r"dollar_(\w+)_children_(\d+)_(\w+)").unwrap();
//     for cap in children_pattern.captures_iter(&block_string) {
//         let node_id = &cap[1];
//         let index = &cap[2];
//         let _property = &cap[3];
//         children.push((node_id.to_string(), index.parse::<u32>().unwrap()));
//     }
//     children
// }

pub fn transform_dollar_this_parent_syntax(
    this_id: &str,
    parent_id: &str,
    code: &str,
    properties_mapping: &HashMap<String, AbstractValue>,
    node_hierarchy_map: &Vec<(String, Vec<String>)>,
) -> String {

    //println!("transform_dollar_this_parent_syntax");

    if code.contains("dollar_this") || code.contains("dollar_parent") {
        println!("transform_dollar_this_parent_syntax: code contains $.this or $.parent");
    }

    // replace $.this and $.parent with dollar_this and dollar_parent
    let code = code
        .replace("dollar_this", &format!("$.{}", this_id))
        .replace("dollar_parent", &format!("$.{}", parent_id));
    transform_dollar_syntax(&code, properties_mapping, node_hierarchy_map)
}

pub fn transform_dollar_syntax(
    code: &str,
    properties_mapping: &HashMap<String, AbstractValue>,
    node_hierarchy_map: &Vec<(String, Vec<String>)>,
) -> String {

    println!("transform_dollar_syntax");

    // Only transform if there are actually $ expressions
    if !code.contains("$.") {
        return code.to_string();
    }

    // replace $.this and $.parent with dollar_this and dollar_parent
    let code = code
        .replace("$.this", "dollar_this")
        .replace("$.parent", "dollar_parent")
        .replace("childrens [", "childrens[");
    let mut result = code.trim().to_string();

    // Handle $.node.children_count and $.node.children[i].prop first
    // $.node.children_count is get only
    let children_count_pattern =
        Regex::new(r"\$\.([a-zA-Z_][a-zA-Z0-9_]*)\.children_count\b").unwrap();
    result = children_count_pattern.replace_all(&result, |caps: &regex::Captures| {
        let node_id = &caps[1];
        format!("engine.get_children_count_by_id(\"{}\")", node_id)
    }).to_string();

    println!("result: {}", result);

    let children_pattern = Regex::new( r"\$\.\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\.childrens\[\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\]\s*\.\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*([^;]+);").unwrap();
    result = children_pattern.replace_all(&result, |caps: &regex::Captures| {
        //print

        let node_id = &caps[1];
        let index = &caps[2];
        let property = &caps[3];
        let value = &caps[4];

        println!("pattern match !!!! $.{}.childrens[{}].{}", node_id, index, property);

        // ok, i'm so dumb, but here i can't just replace $.node.childrens[i].prop with $.an_id.prop
        // because, i depend of the runtime execution...

        // so i have to find the id of the child at index i, but at runtime.
        // i can do that, but then i need to know the prop type...
        // but to know that i need to find it in the properties_mapping at compile time
        // a bit disapointing.

        // ok, thinking nasty, i could resurrect the $.this.childrens[i].top_margin:number = y; notation
        // but i don't want to do that

        // or think even nastier, i could just replace the whole bloc with a :
        /*
        
            if i == 0 {
                $.this.childrens[0].top_margin = y;
            }
            else if i == 1 {
                $.this.childrens[1].top_margin = y;
            }
            ...
            else {
                $.this.childrens[150].top_margin = y;
            }

            where 151 is the number of children
        
         */

        // let's go

        let code = node_hierarchy_map
            .iter()
            .find(|(curr_id, _)| curr_id == node_id)
            .map(|(_, v)| {
                v.iter().enumerate()  
                    .map(|(idx, child_id)| {
                        match properties_mapping.get(&format!("{}.{}", child_id, property)) {
                            Some(_) => { format!("if {index} == {idx} {{ println!(\"index is {index}\"); $.{child_id}.{property} = {value}; }}") },
                            None => {
                                println!("property {} not found", format!("{}.{}", child_id, property));
                                "".to_string()
                            }
                        } 
                        
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap();



        // let out = String::new();

        // for i in 0..number_of_childs {
        //     let child_id = node_hierarchy_map
        //         .iter()
        //         .find(|(curr_id, _)| curr_id == node_id)
        //         .map(|(_, v)| v[i].clone())
        //         .unwrap();

        code

    }).to_string();

    // $.repeater_example.childrens[0].text must be translated to childrens[0] id .text (ex if first child of repeater is test01 : test01.text)
    // for that we need to be able to know the id of the child at an index of a node
    // we need a node_hierarchy_map
    let children_pattern = Regex::new(r"\$\.([a-zA-Z_][a-zA-Z0-9_]*)\.childrens\[(\d+)\]\.([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    result = children_pattern.replace_all(&result, |caps: &regex::Captures| {
        let node_id = &caps[1];
        let index = &caps[2];
        let property = &caps[3];

        // find the child id from the node_hierarchy_map
        // map item are like this: 
        /*
            (root, vec(repeater_example))
            (repeater_example, vec(generated_id_0, generated_id_1, etc...))
            etc...
        */

        println!("pattern match !!!! 2");

        let child_id = node_hierarchy_map
            .iter()
            .find(|(parent_id, _)| parent_id == node_id)
            .unwrap()
            .1
            .get(index.parse::<usize>().unwrap())
            .unwrap();

        println!("$.{}.{}", child_id, property);

        format!("$.{}.{}", child_id, property)

    }).to_string();

    // translation to engine calls

    // Handle compound assignments first: $.node.prop += value; ([+\-*/])=
    let compound_assign_pattern = Regex::new(
        r"\$\.([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)\s*([+\-*/])=\s*([^;]+)\s*;",
    )
    .unwrap();
    result = compound_assign_pattern.replace_all(&result, |caps: &regex::Captures| {
        let node_id = &caps[1];
        let property = &caps[2];
        let operator = &caps[3];
        let value = &caps[4].trim();

        let abstract_value = properties_mapping.get(&format!("{}.{}", node_id, property));

        match Some(abstract_value) {
            Some(Some(AbstractValue::String(_))) => {
                if operator != "+" {
                    panic!("Invalid operator '{}' for string property '{}.{}'. Only '+=' is allowed for strings.", operator, node_id, property);
                }
                // format!("set_string!(engine, {}, {}, format!(\"{{}}{{}}\", get_string!(engine, {}, {}), {}));", 
                //     node_id, property, node_id, property, value)
                format!("engine.set_property_of_node(\"{}\", \"{}\", AbstractValue::String(format!(\"{{}}{{}}\", engine.get_string_property_of_node(\"{}\", \"{}\", \"\"), {}));", 
                    node_id, property, node_id, property, value)
            },
            Some(Some(AbstractValue::Bool(_))) => {
                panic!("Compound assignments are not supported for boolean properties '{}.{}'.", node_id, property);
            },
            Some(Some(AbstractValue::Color(_))) => {
                panic!("Compound assignments are not supported for color properties '{}.{}'.", node_id, property);
            },
            Some(Some(AbstractValue::Number(_))) => {
                // format!("set_number!(engine, {}, {}, get_number!(engine, {}, {}) {} {});", 
                //     node_id, property, node_id, property, operator, value)
                format!("engine.set_property_of_node(\"{}\", \"{}\", AbstractValue::Number(engine.get_number_property_of_node(\"{}\", \"{}\", 0.0) {} {}));", 
                    node_id, property, node_id, property, operator, value)
            }
            _ => {
                panic!("Can't find property '{}.{}'.", node_id, property);
            }
        }
    }).to_string();

    // Handle simple assignments: $.node.prop = value;
    // Match = but not ==, !=, <=, >=, +=, -=, *=, /=
    let assign_pattern = Regex::new(
        r"\$\.([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*([^=;][^;]*)\s*;",
    )
    .unwrap();
    result = assign_pattern
        .replace_all(&result, |caps: &regex::Captures| {
            println!("assign_pattern {}", caps.get(0).unwrap().as_str());
            let node_id = &caps[1];
            let property = &caps[2];
            let value = &caps[3].trim();

            let abstract_value = properties_mapping.get(&format!("{}.{}", node_id, property));
            match Some(abstract_value) {
                Some(Some(AbstractValue::String(_))) => {
                    //format!("set_string!(engine, {}, {}, {});", node_id, property, value)
                    format!("engine.set_property_of_node(\"{}\", \"{}\", AbstractValue::String({}));", node_id, property, value)
                }
                Some(Some(AbstractValue::Bool(_))) => {
                    //format!("set_bool!(engine, {}, {}, {});", node_id, property, value)
                    format!("engine.set_property_of_node(\"{}\", \"{}\", AbstractValue::Bool({}));", node_id, property, value)
                }
                Some(Some(AbstractValue::Color(_))) => {
                    //format!("set_color!(engine, {}, {}, {});", node_id, property, value)
                    format!("engine.set_property_of_node(\"{}\", \"{}\", AbstractValue::Color({}));", node_id, property, value)
                }
                Some(Some(AbstractValue::Number(_))) => {
                    //("set_number!(engine, {}, {}, {});", node_id, property, value)
                    format!("engine.set_property_of_node(\"{}\", \"{}\", AbstractValue::Number({}));", node_id, property, value)
                }
                _ => {
                    panic!("Can't find property '{}.{}'.", node_id, property);
                }
            }
        })
        .to_string();

    // Handle regular read operations: $.node.prop (in expressions)
    // Be more careful to only match standalone expressions, not inside strings
    let dollar_pattern =
        Regex::new(r"\$\.([a-zA-Z_][a-zA-Z0-9_]*)\.([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();
    result = dollar_pattern
        .replace_all(&result, |caps: &regex::Captures| {
            println!("get_pattern {}", caps.get(0).unwrap().as_str());
            let node_id = &caps[1];
            let property = &caps[2];

            let abstract_value = properties_mapping.get(&format!("{}.{}", node_id, property));
            match Some(abstract_value) {
                Some(Some(AbstractValue::String(_))) => {
                    //format!("get_string!(engine, {}, {})", node_id, property)
                    format!("engine.get_string_property_of_node(\"{}\", \"{}\", \"\".to_string())", node_id, property)
                }
                Some(Some(AbstractValue::Bool(_))) => {
                    //format!("get_bool!(engine, {}, {})", node_id, property)
                    format!("engine.get_bool_property_of_node(\"{}\", \"{}\", false)", node_id, property)
                }
                Some(Some(AbstractValue::Color(_))) => {
                    //format!("get_color!(engine, {}, {})", node_id, property)
                    format!("engine.get_color_property_of_node(\"{}\", \"{}\", RED)", node_id, property)
                }
                Some(Some(AbstractValue::Number(_))) => {
                    //format!("get_number!(engine, {}, {})", node_id, property)
                    format!("engine.get_number_property_of_node(\"{}\", \"{}\", 0.0)", node_id, property)
                }
                _ => {
                    panic!("Can't find property '{}.{}'.", node_id, property);
                }
            }
        })
        .to_string();

    result
}

/* 
#[cfg(test)]
mod tests {
    use rml_core::prelude::DARKGRAY;

    use super::*;
    use std::collections::HashMap;

    fn mapping() -> HashMap<String, AbstractValue> {
        use AbstractValue::*;
        let mut m = HashMap::new();
        m.insert("node.str".into(), String("".into()));
        m.insert("node.num".into(), Number(0.0));
        m.insert("node.bool".into(), Bool(false));
        m.insert("node.color".into(), Color(DARKGRAY));
        m
    }

    // ---------------------------------------------------
    // 1. SIMPLE ASSIGNMENTS
    // ---------------------------------------------------
    #[test]
    fn test_simple_assignment_number() {
        let code = "$.node.num = 5;";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(result, "set_number!(engine, node, num, 5);");
    }

    #[test]
    fn test_simple_assignment_string() {
        let code = "$.node.str = \"hello\";";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(result, "set_string!(engine, node, str, \"hello\");");
    }

    #[test]
    fn test_simple_assignment_bool() {
        let code = "$.node.bool = true;";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(result, "set_bool!(engine, node, bool, true);");
    }

    #[test]
    fn test_simple_assignment_color() {
        let code = "$.node.color = red;";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(result, "set_color!(engine, node, color, red);");
    }

    // ---------------------------------------------------
    // 2. COMPARISONS (==) SHOULD NOT BE TRANSFORMED
    // ---------------------------------------------------
    #[test]
    fn test_comparison_not_transformed() {
        let code = "if ($.node.num == 3) {}";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(result, "if (get_number!(engine, node, num) == 3) {}");
    }

    #[test]
    fn test_comparison_with_strings_not_transformed() {
        let code = "if ($.node.str == \"ok\") {}";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(result, "if (get_string!(engine, node, str) == \"ok\") {}");
    }

    // ---------------------------------------------------
    // 3. COMPOUND ASSIGNMENTS
    // ---------------------------------------------------
    #[test]
    fn test_compound_plus_number() {
        let code = "$.node.num += 2;";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(
            result,
            "set_number!(engine, node, num, get_number!(engine, node, num) + 2);"
        );
    }

    #[test]
    fn test_compound_plus_string() {
        let code = "$.node.str += \" world\";";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(
            result,
            "set_string!(engine, node, str, format!(\"{}{}\", get_string!(engine, node, str), \" world\"));"
        );
    }

    #[test]
    #[should_panic(expected = "Invalid operator '-' for string property")]
    fn test_compound_wrong_operator_string() {
        transform_dollar_syntax("$.node.str -= \"bad\";", &mapping());
    }

    #[test]
    #[should_panic(expected = "Compound assignments are not supported for boolean properties")]
    fn test_compound_bool_invalid() {
        transform_dollar_syntax("$.node.bool += true;", &mapping());
    }

    // ---------------------------------------------------
    // 4. READ OPERATIONS
    // ---------------------------------------------------
    #[test]
    fn test_read_number() {
        let code = "x = $.node.num + 1;";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(result, "x = get_number!(engine, node, num) + 1;");
    }

    #[test]
    fn test_read_string() {
        let code = "print($.node.str);";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(result, "print(get_string!(engine, node, str));");
    }

    #[test]
    fn test_read_multiple() {
        let code = "$.node.num + $.node.num * $.node.num";
        let result = transform_dollar_syntax(code, &mapping());
        assert_eq!(
            result,
            "get_number!(engine, node, num) + get_number!(engine, node, num) * get_number!(engine, node, num)"
        );
    }

    // ---------------------------------------------------
    // 5. UNKNOWN PROPERTY
    // ---------------------------------------------------
    #[test]
    #[should_panic(expected = "Can't find property")]
    fn test_unknown_property() {
        transform_dollar_syntax("$.x.y = 10;", &mapping());
    }

    // ---------------------------------------------------
    // 5. UNKNOWN PROPERTY
    // ---------------------------------------------------
    #[test]
    fn test_real_01_property() {
        let code = "fn compute_font_size() { if $.node.num == 0.0 { $.node.str = \"Click Me\".to_string(); } }";
        let result = transform_dollar_syntax(code, &mapping());
        println!("Result: {}", result);
        assert_eq!(
            result,
            "fn compute_font_size() { if get_number!(engine, node, num) == 0.0 { set_string!(engine, node, str, \"Click Me\".to_string()); } }"
        );
    }
}
    */

pub fn inject_engine_text_based(
    input: &str,
    engine_str: &str,
    definition: bool,
    mutable: bool,
    functions: &Vec<String>,
) -> String {
    // prepare what we should inject OUTSIDE callback blocks
    let injected_normal = if definition {
        format!("&mut {engine_str}")
    } else if mutable {
        format!("&mut {engine_str}")
    } else {
        format!("{engine_str}")
    };

    // inside a callback block we always inject "engine"
    let injected_callback = "engine".to_string();

    let mut output = String::new();

    let mut in_callback = false;
    let mut brace_depth: i32 = 0;

    for line in input.lines() {
        let mut modified_line = line.to_string();

        // detect start of callback
        if line.contains("engine.add_callback") && line.contains("move | engine |") {
            in_callback = true;
            // we expect an opening brace, but we count braces
            // anyway to remain robust.
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;
            output.push_str(&modified_line);
            output.push('\n');
            continue;
        }

        // If inside callback, keep track of braces to know when we exit
        if in_callback {
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;
            if brace_depth <= 0 {
                in_callback = false;
            }
        }

        for func in functions {
            let def_pattern = format!("fn {func}(");

            if line.contains(&def_pattern) {
                // Function definition – skip
                continue;
            }

            // Function call injection
            let call_pattern = format!("{func}(");

            if modified_line.contains(&call_pattern) {
                let replacement = if in_callback {
                    format!("{func}({injected_callback}")
                } else {
                    format!("{func}({injected_normal}")
                };

                modified_line = modified_line.replace(&call_pattern, &replacement);
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
                        let has_engine = call
                            .args
                            .iter()
                            .any(|arg| matches!(arg, Expr::Path(p) if p.path.is_ident("engine")));

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

pub fn find_related_property_for_binding(
    id: String,
    property: String,
    block_string: String,
) -> Vec<(String, String)> {

    println!("find_related_property_for_binding");
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

        if trimmed_line.contains("get_value!")
            || trimmed_line.contains("get_number!")
            || trimmed_line.contains("get_string!")
            || trimmed_line.contains("get_bool!")
            || trimmed_line.contains("get_color!")
            || trimmed_line.contains("get_computed_x!")
            || trimmed_line.contains("get_computed_y!")
            || trimmed_line.contains("get_computed_width!")
            || trimmed_line.contains("get_computed_height!")
        {
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
        } else if trimmed_line.contains("get_number_property_of_node")
            || trimmed_line.contains("get_string_property_of_node")
            || trimmed_line.contains("get_bool_property_of_node")
            || trimmed_line.contains("get_color_property_of_node")
            || trimmed_line.contains("get_property_of_node")
        {
            println!("trimmed {trimmed_line}");
            // Parse method calls like engine.get_number_property_of_node("node_name", "property_name", default)
            if let Some(start) = trimmed_line.find('(') {
                if let Some(end) = trimmed_line.rfind(')') {
                    let params = &trimmed_line[start + 1..end];
                    let parts: Vec<&str> = params.split(',').map(|s| s.trim()).collect();

                    if parts.len() >= 2 {
                        let node_name = parts[0].trim().trim_matches('"');
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

    println!("related {related_properties:?}");
    related_properties
}
