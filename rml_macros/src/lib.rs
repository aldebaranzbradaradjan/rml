use proc_macro::TokenStream;
use quote::{quote};
use syn::parse::{ParseStream};

mod structs;
use structs::*;
mod macros;
mod format;
use format::*;

#[proc_macro]
pub fn rml(input: TokenStream) -> TokenStream {
    
    //let input_string = input.to_string();
    // //let input_string = transform_dollar_syntax(&input_string);
    
    // First parse
    //let transformed_input: TokenStream = input_string.parse().unwrap();
    let mut res = RmlParser::empty();
    match syn::parse::Parser::parse(|input: ParseStream| {
        res = RmlParser::parse_with_path(input, "".to_string(), true).unwrap();
        Ok(RmlParser::empty())
    }, input.clone()) {
        Ok(r) => r,
        Err(_e) => { RmlParser::empty() }
    };

    // // now we need equivalent process to parsed_node.generate_with_components(&components);
    // // but only to develop children components, and be sure to use a deterministic way to rename items (a global counter should do the tricks)
    // // after that we could be able to map a list of property name with theirs types
    // let (mut parsed_node, components) = (res.root_node, res.components);
    // let properties_mapping = parsed_node.pre_generate_with_components_and_counter(&components, &mut 0);

    // println!("Res : {:#?}", properties_mapping);

    // // we have the struct of the application, and can infer property type, and use it in transform_dollar_syntax

    // // transform the input to replace $ syntax before parsing
    // let input_string = transform_dollar_syntax(&input_string, &properties_mapping);

    // //println!("Transformed input: {}", input_string);
    
    // // Parse the transformed input
    // let transformed_input: TokenStream = input_string.parse().unwrap();
    // let res = syn::parse::Parser::parse(|input: ParseStream| {
    //     RmlParser::parse_with_path(input, "".to_string(), false)
    // }, transformed_input.clone()).unwrap();

    // let (mut parsed_node, components) = (res.root_node, res.components);
    
    // let generated = parsed_node.generate_with_components_and_counter(&components, &mut 0, &properties_mapping);
    // let generated_node = generated.1;
    // let generated_functions = generated.2;
    // let generated_initializer = generated.3;

    // let functions_name: Vec<String> = parsed_node
    //     .functions
    //     .iter()
    //     .map(|f| f.sig.ident.to_string())
    //     .collect();

    // let generated_initializer = generated_initializer.to_string();
    // //let generated_initializer = format_code(&generated_initializer);
    // let generated_initializer = inject_engine_text_based(&generated_initializer, "engine", false, true, &functions_name);
    // let generated_initializer = generated_initializer.parse::<proc_macro2::TokenStream>().unwrap();

    // let generated_functions = generated_functions.to_string();
    // let generated_functions = format_code(&generated_functions);
    // let generated_functions = inject_engine_text_based(&generated_functions, "engine", true, true, &functions_name);
    // let generated_functions = generated_functions.parse::<proc_macro2::TokenStream>().unwrap();

    // let generated_node = generated_node.to_string();
    // let generated_node = inject_engine_text_based(&generated_node, "engine", false, false, &functions_name);
    // let generated_node = generated_node.parse::<proc_macro2::TokenStream>().unwrap();

    // let result = quote! {
    //     {
    //         let mut engine = RmlEngine::new();
    //         #generated_node;
    //         #generated_functions
    //         #generated_initializer
    //         engine
    //     }
    // };


    let result = quote! {
        RmlEngine::new()
    };


    TokenStream::from(result)
}
