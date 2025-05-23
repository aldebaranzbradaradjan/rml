use macroquad::prelude::*;

use crate::{RmlEngine, Property, AbstractValue, ItemTypeEnum};

use std::collections::VecDeque;

fn compute_position(engine: &RmlEngine, computed_parent_x : f32, computed_parent_y : f32, node_id: &str) -> (f32, f32) {
    let x = engine.get_number_property_of_node(node_id, "x", 0.0);
    let y = engine.get_number_property_of_node(node_id, "y", 0.0);
    let width = engine.get_number_property_of_node(node_id, "width", 0.0);
    let height = engine.get_number_property_of_node(node_id, "height", 0.0);
    
    // Get the parent node
    let (parent_width, parent_height) = match engine.get_node_by_id(node_id) {
        None => {
            return (0.0, 0.0);
        }
        Some(parent) => {
            // Get the parent node's properties
            let id = parent.id.as_str();
            // Get the parent node's properties
            let parent_width = engine.get_number_property_of_node(id, "width", 0.0);
            let parent_height = engine.get_number_property_of_node(id, "height", 0.0);
            (parent_width, parent_height)
        }
    };

    // get the node anchors values
    // anchors: top | bottom | left | right | horizontal_center | vertical_center | center | fill
    let anchors = engine.get_string_property_of_node(node_id, "anchors", "".to_string());

    // Compute the position of the node
    let x = computed_parent_x + x;
    let y = computed_parent_y + y;

    (x, y)
}

// fn compute_size(engine: &RmlEngine, node_id: &str) -> (f32, f32) {
//     let width = engine.get_number_property_of_node(node_id, "width", 0.0);
//     let height = engine.get_number_property_of_node(node_id, "height", 0.0);
//     (width, height)
// }

pub fn draw_childs(engine : &RmlEngine, node_id : &str, computed_parent_x : f32, computed_parent_y : f32) {
    // Draw the elements in the node
    if let Some(childs) = engine.get_children_by_id(node_id) {
        if childs.is_empty() { return; }
        let mut childs_computed_x = VecDeque::new();
        let mut childs_computed_y = VecDeque::new();

        for node in childs.clone() {
            // let x = engine.get_number_property_of_node(node.id.as_str(), "x", 0.0);
            // let y = engine.get_property_of_node(node.id.as_str(), "y", 0.0, |v| v.to_number().map(|n| n as f32));
            // let mut x = x;
            // let mut y = y;

            // Compute the position of the node
            let (computed_x, computed_y) = compute_position(engine, computed_parent_x, computed_parent_y, node.id.as_str());

            childs_computed_x.push_back(computed_x);
            childs_computed_y.push_back(computed_y);

            if(node.node_type == ItemTypeEnum::Rectangle) {
                let width = engine.get_number_property_of_node(node.id.as_str(), "width", 0.0);
                let height = engine.get_number_property_of_node(node.id.as_str(), "height", 0.0);
                let color = engine.get_color_property_of_node(node.id.as_str(), "color", WHITE);
                draw_rectangle(computed_x, computed_y, width, height, color);
            }
            else if(node.node_type == ItemTypeEnum::Text) {
                let text = engine.get_string_property_of_node(node.id.as_str(), "text", "".to_string());
                let color = engine.get_color_property_of_node(node.id.as_str(), "color", WHITE);
                let font_size = engine.get_number_property_of_node(node.id.as_str(), "font_size", 20.0);
                draw_text_ex(&text, computed_x, computed_y, TextParams {
                    font_size: font_size as u16,
                    color,
                    ..Default::default()
                });
            }
        }

        // it seems that the order of the childrens can be different between launch
        for node in childs {
            // Draw the children of the node
            let x = childs_computed_x.pop_front().unwrap();
            let y = childs_computed_y.pop_front().unwrap();
            draw_childs(engine, node.id.as_str(), x, y);
        }
    }
}


