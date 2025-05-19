use macroquad::prelude::*;

use crate::{RmlEngine, Property, AbstractValue, ItemTypeEnum};

pub fn draw_childs(engine : &RmlEngine, node_id : &str, parent_x : f32, parent_y : f32) {
    // Draw the elements in the node
    if let Some(childs) = engine.get_children_by_id(node_id) {
        if childs.is_empty() { return; }
        let mut childs_computed_x = Vec::new();
        let mut childs_computed_y = Vec::new();

        for node in childs.clone() {
            let x = engine.get_number_property_of_node(node.id.as_str(), "x", 0.0);
            let y = engine.get_property_of_node(node.id.as_str(), "y", 0.0, |v| v.to_number().map(|n| n as f32));

            childs_computed_x.push(parent_x + x);
            childs_computed_y.push(parent_y + y);

            println!("node: {:#?}", node);
            println!("childs_computed_x: {:#?}", childs_computed_x);
            println!("childs_computed_y: {:#?}", childs_computed_y);

            if(node.node_type == ItemTypeEnum::Rectangle) {
                let width = engine.get_number_property_of_node(node.id.as_str(), "width", 0.0);
                let height = engine.get_number_property_of_node(node.id.as_str(), "height", 0.0);
                let color = engine.get_color_property_of_node(node.id.as_str(), "color", WHITE);
                draw_rectangle(parent_x + x, parent_y + y, width, height, color);
            }
            else if(node.node_type == ItemTypeEnum::Text) {
                let text = engine.get_string_property_of_node(node.id.as_str(), "text", "".to_string());
                let color = engine.get_color_property_of_node(node.id.as_str(), "color", WHITE);
                let font_size = engine.get_number_property_of_node(node.id.as_str(), "font_size", 20.0);
                draw_text_ex(&text, parent_x + x, parent_y + y, TextParams {
                    font_size: font_size as u16,
                    ..Default::default()
                });
            }
        }

        // it seems that the order of the childrens can be different between launch
        for node in childs {
            // Draw the children of the node
            draw_childs(engine, node.id.as_str(), childs_computed_x.pop().unwrap(), childs_computed_y.pop().unwrap());
        }
    }
}


