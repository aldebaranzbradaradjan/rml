use macroquad::prelude::*;

use crate::{RmlEngine, Property, AbstractValue, ItemTypeEnum};

use std::collections::VecDeque;

fn compute_geometry(engine: &RmlEngine, computed_parent_x : f32, computed_parent_y : f32, node_id: &str) -> (f32, f32, f32, f32) {
    let mut x = engine.get_number_property_of_node(node_id, "x", 0.0);
    let mut y = engine.get_number_property_of_node(node_id, "y", 0.0);
    let mut width = engine.get_number_property_of_node(node_id, "width", 0.0);
    let mut height = engine.get_number_property_of_node(node_id, "height", 0.0);
    
    // Get the parent node
    let (parent_width, parent_height) = match engine.get_parent_by_id(node_id) {
        None => {
            (0.0, 0.0)
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
    let anchors = anchors.split("__").collect::<Vec<&str>>();
    for anchor in anchors.clone() {
        // Compute the position of the node
        match anchor {
            "top" => y = 0.,
            "bottom" => y = parent_height - height,
            "left" => x = 0.,
            "right" => x = parent_width - width,
            "horizontal_center" => x = (parent_width / 2.0) - (width / 2.0),
            "vertical_center" => y = (parent_height / 2.0) - (height / 2.0),
            "center" => {
                x = (parent_width / 2.0) - (width / 2.0);
                y = (parent_height / 2.0) - (height / 2.0);
            }
            _ => {}
        }
    }

    // if fill or top && bottom are set, we need to compute the height to shrink the item
    if anchors.contains(&"fill") || (anchors.contains(&"top") && anchors.contains(&"bottom")) {
        y = 0.; height = parent_height;
    }
    // if fill or left && right are set, we need to compute the width to shrink the item
    if anchors.contains(&"fill") || (anchors.contains(&"left") && anchors.contains(&"right")) {
        x = 0.; width = parent_width;
    }

    // Get the margins values
    let margins = engine.get_number_property_of_node(node_id, "margins", 0.0);
    let mut left_margin = engine.get_number_property_of_node(node_id, "left_margin", margins);
    let mut right_margin = engine.get_number_property_of_node(node_id, "right_margin", margins);
    let mut top_margin = engine.get_number_property_of_node(node_id, "top_margin", margins);
    let mut bottom_margin = engine.get_number_property_of_node(node_id, "bottom_margin", margins);
    
    if !anchors.contains(&"left") {
        left_margin = 0.;
    }
    if !anchors.contains(&"right") {
        right_margin = 0.;
    }
    if !anchors.contains(&"top") {
        top_margin = 0.;
    }
    if !anchors.contains(&"bottom") {
        bottom_margin = 0.;
    }
    if anchors.contains(&"vertical_center") {
        top_margin = 0.;
        bottom_margin = 0.;
    }
    if anchors.contains(&"horizontal_center") {
        left_margin = 0.;
        right_margin = 0.;
    }
    if anchors.contains(&"center") {
        top_margin = 0.;
        bottom_margin = 0.;
        left_margin = 0.;
        right_margin = 0.;
    }
    if anchors.contains(&"fill") {
        top_margin = 0.;
        bottom_margin = 0.;
        left_margin = 0.;
        right_margin = 0.;
    }

    // Compute the width and height of the node
    width = width - right_margin;
    height = height - bottom_margin;

    // Compute the position of the node
    x = computed_parent_x + x + left_margin;
    y = computed_parent_y + y + top_margin;

    (x, y, width, height)
}

pub fn draw_childs(engine : &RmlEngine, node_id : &str, computed_parent_x : f32, computed_parent_y : f32) {
    // Draw the elements in the node
    if let Some(childs) = engine.get_children_by_id(node_id) {
        if childs.is_empty() { return; }
        let mut childs_computed_x = VecDeque::new();
        let mut childs_computed_y = VecDeque::new();

        for node in childs.clone() {
            // Compute the position of the node
            let (computed_x,
                computed_y,
                computed_width,
                computed_height) = compute_geometry(engine, computed_parent_x, computed_parent_y, node.id.as_str());

            childs_computed_x.push_back(computed_x);
            childs_computed_y.push_back(computed_y);

            if node.node_type == ItemTypeEnum::Rectangle {
                let width = computed_width;//engine.get_number_property_of_node(node.id.as_str(), "width", 0.0);
                let height = computed_height;//engine.get_number_property_of_node(node.id.as_str(), "height", 0.0);
                let color = engine.get_color_property_of_node(node.id.as_str(), "color", WHITE);
                draw_rectangle(computed_x, computed_y, width, height, color);
            }
            else if node.node_type == ItemTypeEnum::Text {
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


