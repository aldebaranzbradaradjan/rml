use macroquad::prelude::*;

use crate::{RmlEngine};

pub fn draw_element(engine : &RmlEngine, node_id : &str) {
    // Draw the elements in the node
    if let Some(childs) = engine.get_children_by_id(node_id) {
        if childs.is_empty() { return; }

        for node in childs {
            //if(node.type == "Rectangle") {
                let x = engine.get_number_property_of_node(node.id.as_str(), "x", 0.0);
                let y = engine.get_property_of_node(node.id.as_str(), "y", 0.0, |v| v.to_number().map(|n| n as f32)); //ps.get_number_property_of_node(node.id.as_str(), "y", 0.0);
                let width = engine.get_number_property_of_node(node.id.as_str(), "width", 0.0);
                let height = engine.get_number_property_of_node(node.id.as_str(), "height", 0.0);
                let color = engine.get_color_property_of_node(node.id.as_str(), "color", WHITE);
                //println!("y is {}", y);
    
                //println!("Drawing rectangle at ({}, {}), size ({}, {})", x, y, width, height);
                draw_rectangle(x, y, width, height, color);
            //}
            
            draw_element(engine, node.id.as_str());
        }
    }
}


