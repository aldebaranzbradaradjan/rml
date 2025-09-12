use proc_macro::TokenStream;
use quote::{quote};
use rml_core::prelude::warn;
use syn::parse::{ParseStream};

mod structs;
use structs::*;
mod macros;
mod format;
use format::*;

#[proc_macro]
pub fn rml(input: TokenStream) -> TokenStream {
    // First, transform the input to replace $ syntax before parsing
    let input_string = input.to_string();
    let transformed_string = transform_dollar_syntax(&input_string);
    
    // Parse the transformed input
    let transformed_input: TokenStream = match transformed_string.parse() {
        Ok(tokens) => tokens,
        Err(_) => { warn!("Failed to transform input"); input }, // Fallback to original if transformation fails
    };

    let res = syn::parse::Parser::parse(|input: ParseStream| {
        RmlParser::parse_with_path(input, "".to_string())
    }, transformed_input.clone()).unwrap();

    let (parsed_node, components) = (res.root_node, res.components);
    
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
