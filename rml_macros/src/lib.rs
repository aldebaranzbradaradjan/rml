use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, Lit, Token};
use uuid::Uuid;

use rml_core::{AbstractValue};

#[proc_macro]
pub fn rml(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as RmlNode);
    let generated = parsed.generate();
    let generated_code = generated.0;
    let result = quote! {
        {
            let mut engine = RmlEngine::new();
            #generated_code; // tree computed here
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
        Lit::Str(lit_str) => Some(lit_str.value()), // Extract the value from a LitStr
        Lit::ByteStr(lit_byte_str) => {
            Some(String::from_utf8_lossy(&lit_byte_str.value()).to_string())
        }
        Lit::Char(lit_char) => Some(lit_char.value().to_string()),
        Lit::Int(lit_int) => Some(lit_int.base10_digits().to_string()), // Convert integer to its string representation
        Lit::Float(lit_float) => Some(lit_float.base10_digits().to_string()), // Convert float to its string representation
        Lit::Bool(lit_bool) => Some(lit_bool.value.to_string()), // Convert boolean to "true" or "false"
        Lit::Byte(lit_byte) => Some(format!("{:?}", lit_byte.value())), // Convert byte to a string
        // Add additional cases if needed for other literal types.
        _ => None, // If it's an unsupported literal type
    }
}

// fn value_to_property(value: &Value) -> Property {
//     match value {
//         Value::Lit(literal) => match literal {
//             Lit::Str(lit_str) => Property::new(AbstractValue::String(lit_str.value())),
//             Lit::ByteStr(lit_byte_str) => {
//                 Property::new(AbstractValue::String(String::from_utf8_lossy(&lit_byte_str.value()).to_string()))
//             }
//             Lit::Char(lit_char) => Property::new(AbstractValue::String(lit_char.value().to_string())),
//             Lit::Int(lit_int) => {
//                 let number: f32 = lit_int.base10_parse().unwrap_or_default();
//                 Property::new(AbstractValue::Number(number))
//             }
//             Lit::Float(lit_float) => {
//                 let number: f32 = lit_float.base10_parse().unwrap_or_default();
//                 Property::new(AbstractValue::Number(number))
//             }
//             Lit::Bool(lit_bool) => Property::new(AbstractValue::Bool(lit_bool.value)),
//             Lit::Byte(lit_byte) => Property::new(AbstractValue::Number(lit_byte.value() as f32)),
//             _ => Property::new(AbstractValue::Null), // Handle unexpected or unsupported literal types
//         },
//         Value::Ident(ident) => Property::new(AbstractValue::String(ident.to_string())),
//     }
// }

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
            _ => AbstractValue::Null, // Handle unexpected or unsupported literal types
        },
        Value::Ident(ident) => AbstractValue::String(ident.to_string()),
        Value::Block(_block) => {
            // Handle block values if needed
            AbstractValue::Null // Placeholder for block handling
        }
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

/// Struct to parse a Node
struct RmlNode {
    _ident: Ident, // unused directly, but allow to parse the node name and syntax
    properties: Vec<(Ident, Value)>,
    children: Vec<RmlNode>,
}

impl Parse for RmlNode {
    /// Parse a `RmlNode` from a proc-macro input stream.
    ///
    /// We expect a sequence of tokens that looks like this:
    ///
    /// Node {
    ///     id: root
    ///     width: 200
    ///     height: 200

    ///     Rectangle {
    ///         x: 50
    ///         width: 100
    ///         height: 100
    ///         color: "green"
    ///     }
    /// }
    ///
    /// The parser will return an error if it encounters any other token.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // get the name of the node (Node ou Rectangle)
        let _ident: Ident = input.parse()?;

        // manage braces
        let content;
        syn::braced!(content in input);

        let mut properties = Vec::new();
        let mut children = Vec::new();

        while !content.is_empty() {
            if content.peek(Ident) && content.peek2(Token![:]) {
                // read a property
                let key: Ident = content.parse()?;
                content.parse::<Token![:]>()?;

                let value: Value;

                if content.peek(Lit) {
                    value = Value::Lit(content.parse()?);
                } else if content.peek(Ident) {
                    value = Value::Ident(content.parse()?);
                } else if content.peek(syn::token::Brace) {
                    let block: syn::Block = content.parse()?;
                    value = Value::Block(block);
                } else {
                    return Err(content.error("Expected literal, identifier or block"));
                }
            
                content.parse::<Token![,]>().ok(); // optional virgule
                properties.push((key, value));
            } else if content.peek(Ident) {
                // we have a child
                let child: RmlNode = content.parse()?;
                children.push(child);
            } else {
                return Err(content.error("Unexpected token"));
            }
        }

        Ok(Self {
            _ident,
            properties,
            children,
        })
    }
}

impl RmlNode {
    fn generate(&self) -> (proc_macro2::TokenStream, String) {
        // use or generate id if not present
        let id = self
            .properties
            .iter()
            .find_map(|(k, v)| if k == "id" { Some(v.to_string()) } else { None })
            .unwrap_or_else(|| Uuid::new_v4().simple().to_string());

        // Generate a temporary variable for the node
        let temp_node = format_ident!("temp_node_{}", id);

        // convert properties in token stream
        let properties: Vec<proc_macro2::TokenStream> = self
            .properties
            .iter()
            .map(|(k, v)| {

                let k_string = k.to_string();

                if k_string.starts_with("on_") && k_string.ends_with("_change") {
                    let observed = k_string.trim_start_matches("on_").trim_end_matches("_change");
                    if let Value::Block(block) = v {
                        quote! {
                            let cb_id = engine.add_callback( |engine| #block);
                            engine.bind_node_property_to_callback( #id, #observed, cb_id);
                        }
                    } else {
                        quote! {} // fallback ou erreur
                    }
                } else {
                    let value = value_to_abstract_value(v);
                    quote! {
                        let prop_id = engine.add_property(Property::new( #value ));
                        engine.add_property_to_node(#temp_node, stringify!(#k).to_string() , prop_id);
                    }
                }
            })
            .collect();

        // generate code for each child
        let child_results: Vec<(proc_macro2::TokenStream, String)> = self
            .children
            .iter()
            .map(|child| child.generate())
            .collect();

        // get token stream parts
        let child_code: Vec<proc_macro2::TokenStream> = child_results
            .iter()
            .map(|(code, _)| code.clone())
            .collect();

        // get temporary variable names
        let child_temp_nodes: Vec<proc_macro2::TokenStream> = child_results
            .iter()
            .map(|(_, id)| {
                let child_temp_var = format_ident!("temp_node_{}", id);
                quote! { #child_temp_var }
            })
            .collect();

        // generate current node
        let node_code = quote! {
            let #temp_node = engine.add_node(
                #id.to_string(),
                HashMap::new(),
            ).unwrap();

            #(#properties)*

            #(
                #child_code;
                engine.add_child(#temp_node, #child_temp_nodes);
            )*
        };

        

        (node_code, id)
    }
}
