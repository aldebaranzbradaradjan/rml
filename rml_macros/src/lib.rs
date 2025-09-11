use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};

use rml_core::prelude::info;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{parse_macro_input, Ident, Lit, Token, Expr, ExprPath, Member, LitStr};

use uuid::Uuid;

use rml_core::{AbstractValue, ItemTypeEnum};

use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
// use std::path::PathBuf;

use regex::Regex;

mod preparse;
use preparse::preprocess_rml;

mod format;
use format::*;

mod common;
use common::*;

fn parse_component_content(content: &str) -> Result<RmlNode, Box<dyn std::error::Error>> {
    // Parse the content as a TokenStream and then as an RmlNode
    let tokens: proc_macro2::TokenStream = content.parse()?;
    let parsed = syn::parse2::<RmlNode>(tokens)?;
    Ok(parsed)
}

#[proc_macro]
pub fn rml(input: TokenStream) -> TokenStream {
    // First, transform the input to replace $ syntax before parsing
    //let input_string = input.to_string();
    //let input_string = input.to_string();

    
    //let formatted = reformat_rml(&input_string);
    //println!("DEBUG formatted:\n{}", formatted);

    // Step 2: préparser (import + expand components)
    let expanded_string = preprocess_rml(&input);

    info!("DEBUG expanded:\n{}", expanded_string);

    let transformed_string = transform_dollar_syntax(&expanded_string);
    
    // Parse the transformed input
    let transformed_input: TokenStream = match transformed_string.parse() {
        Ok(tokens) => tokens,
        Err(_) => input, // Fallback to original if transformation fails
    };
    
    let parsed_node = parse_macro_input!(transformed_input as RmlNode);

    let generated = parsed_node.generate();
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

