use macroquad::prelude::*;
use crate::{RmlEngine, ItemTypeEnum};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
enum Anchor {
    Top,
    Bottom, 
    Left,
    Right,
    HorizontalCenter,
    VerticalCenter,
    Center,
    Fill,
}

impl Anchor {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "top" => Some(Anchor::Top),
            "bottom" => Some(Anchor::Bottom),
            "left" => Some(Anchor::Left),
            "right" => Some(Anchor::Right),
            "horizontal_center" => Some(Anchor::HorizontalCenter),
            "vertical_center" => Some(Anchor::VerticalCenter),
            "center" => Some(Anchor::Center),
            "fill" => Some(Anchor::Fill),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Margins {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}

impl Margins {
    fn new(engine: &RmlEngine, node_id: &str) -> Self {
        let base_margin = engine.get_number_property_of_node(node_id, "margins", 0.0);
        Self {
            top: engine.get_number_property_of_node(node_id, "top_margin", base_margin),
            bottom: engine.get_number_property_of_node(node_id, "bottom_margin", base_margin),
            left: engine.get_number_property_of_node(node_id, "left_margin", base_margin),
            right: engine.get_number_property_of_node(node_id, "right_margin", base_margin),
        }
    }

    fn apply_anchor_constraints(&mut self, anchors: &[Anchor]) {
        for anchor in anchors {
            match anchor {
                Anchor::HorizontalCenter => {
                    self.left = 0.0;
                    self.right = 0.0;
                }
                Anchor::VerticalCenter => {
                    self.top = 0.0;
                    self.bottom = 0.0;
                }
                Anchor::Center => {
                    self.left = 0.0;
                    self.right = 0.0;
                    self.top = 0.0;
                    self.bottom = 0.0;
                }
                _ => {} // other anchors keep their margins
            }
        }

        // if fill ies specified, don't alter margins
        if anchors.contains(&Anchor::Fill) { return; }

        // special logic: if we don't have the corresponding anchor, no margin
        if !anchors.contains(&Anchor::Left) { self.left = 0.0; }
        if !anchors.contains(&Anchor::Right) { self.right = 0.0; }
        if !anchors.contains(&Anchor::Top) { self.top = 0.0; }
        if !anchors.contains(&Anchor::Bottom) { self.bottom = 0.0; }
    }
}

#[derive(Debug, Clone)]
struct Geometry {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Geometry {
    fn new(engine: &RmlEngine, node_id: &str) -> Self {
        let mut geometry = Self {
            x: engine.get_number_property_of_node(node_id, "x", 0.0),
            y: engine.get_number_property_of_node(node_id, "y", 0.0),
            width: engine.get_number_property_of_node(node_id, "width", 0.0),
            height: engine.get_number_property_of_node(node_id, "height", 0.0),
        };

        // if the node is a text, and don't have a width or height, we need to measure its size
        if let Some(node) = engine.get_node_by_id(node_id) {
            if node.node_type == ItemTypeEnum::Text {
                let text = engine.get_string_property_of_node(node_id, "text", String::new());
                let font_size = engine.get_number_property_of_node(node_id, "font_size", 20.0);
                
                // mesure the text dimensions
                let text_dimensions = measure_text(&text, None, font_size as u16, 1.0);
                if geometry.width == 0.0 {
                    geometry.width = text_dimensions.width;
                }
                if geometry.height == 0.0 {
                    geometry.height = text_dimensions.height;
                }
            }
        }

        geometry
    }

    fn apply_anchors(&mut self, anchors: &[Anchor], parent_size: (f32, f32)) {
        let (parent_width, parent_height) = parent_size;

        for anchor in anchors {
            match anchor {
                Anchor::Top => self.y = 0.0,
                Anchor::Bottom => self.y = parent_height - self.height,
                Anchor::Left => self.x = 0.0,
                Anchor::Right => self.x = parent_width - self.width,
                Anchor::HorizontalCenter => self.x = (parent_width - self.width) / 2.0,
                Anchor::VerticalCenter => self.y = (parent_height - self.height) / 2.0,
                Anchor::Center => {
                    self.x = (parent_width - self.width) / 2.0;
                    self.y = (parent_height - self.height) / 2.0;
                }
                Anchor::Fill => {
                    self.x = 0.0;
                    self.y = 0.0;
                    self.width = parent_width;
                    self.height = parent_height;
                }
            }
        }

        // Gestion des cas spéciaux pour le remplissage
        if anchors.contains(&Anchor::Top) && anchors.contains(&Anchor::Bottom) {
            self.y = 0.0;
            self.height = parent_height;
        }
        if anchors.contains(&Anchor::Left) && anchors.contains(&Anchor::Right) {
            self.x = 0.0;
            self.width = parent_width;
        }
    }

    fn apply_margins(&mut self, margins: &Margins, anchors: &[Anchor], parent_pos: (f32, f32)) {
        let (parent_x, parent_y) = parent_pos;
        
        // fill special case; if fill is specified, also alter the width and height
        if anchors.contains(&Anchor::Fill) {
            self.x = parent_x + self.x + margins.left;
            self.y = parent_y + self.y + margins.top;
            self.width = self.width - margins.left - margins.right;
            self.height = self.height - margins.top - margins.bottom;
        } else {
            self.x = parent_x + self.x + margins.left - margins.right;
            self.y = parent_y + self.y + margins.top - margins.bottom;
        }
    }
}

fn parse_anchors(anchor_string: &str) -> Vec<Anchor> {
    anchor_string
        .split("__")
        .filter_map(|s| Anchor::from_str(s.trim()))
        .collect()
}

fn get_parent_size(engine: &RmlEngine, node_id: &str) -> (f32, f32) {
    match engine.get_parent_by_id(node_id) {
        Some(parent) => {
            let width = engine.get_number_property_of_node(&parent.id, "width", 0.0);
            let height = engine.get_number_property_of_node(&parent.id, "height", 0.0);
            (width, height)
        }
        None => (0.0, 0.0),
    }
}

fn compute_geometry(engine: &RmlEngine, parent_pos: (f32, f32), node_id: &str) -> (f32, f32, f32, f32) {
    let mut geometry = Geometry::new(engine, node_id);
    let parent_size = get_parent_size(engine, node_id);
    
    let anchor_string = engine.get_string_property_of_node(node_id, "anchors", String::new());
    let anchors = parse_anchors(&anchor_string);
    geometry.apply_anchors(&anchors, parent_size);

    let mut margins = Margins::new(engine, node_id);
    margins.apply_anchor_constraints(&anchors);
    geometry.apply_margins(&margins, &anchors, parent_pos);

    (geometry.x, geometry.y, geometry.width, geometry.height)
}

fn draw_text_with_wrap(text: &str, x: f32, y: f32, max_width: f32, text_params: &TextParams) {
    let font_size = text_params.font_size as f32;
    
    if max_width <= 0.0 {
        // if there is no max width, just draw the text at the specified position
        // adding the baseline offset to the y position because macroquad's draw_text_ex uses the baseline offset
        let text_dims = measure_text(&text, None, font_size as u16, 1.0);
        let baseline_offset = text_dims.offset_y; // macroquad fournit cet offset
        let adjusted_y = y + baseline_offset;
        draw_text_ex(text, x, adjusted_y, text_params.clone());
        return;
    }

    let line_height = font_size * 1.2; // space between lines, TODO should be a default value or configurable
    
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };
        
        let test_width = measure_text(&test_line, None, text_params.font_size, 1.0).width;
        
        if test_width <= max_width {
            current_line = test_line;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line = word.to_string();
            } else {
                // the word itself is too long, push it as a new line
                // TODO handle this case better, maybe split the word or truncate it if some parameter is set
                lines.push(word.to_string());
            }
        }
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    
    // draw each line with the adjusted y position
    for (i, line) in lines.iter().enumerate() {
        let text_dims = measure_text(&text, None, font_size as u16, 1.0);
        let baseline_offset = text_dims.offset_y; // macroquad fournit cet offset
        let adjusted_y = y + baseline_offset + (i as f32 * line_height);
        draw_text_ex(line, x, adjusted_y, text_params.clone());
    }
}

pub fn draw_childs(engine: &mut RmlEngine, node_id: &str, parent_pos: (f32, f32)) {
    // Get the node by its ID
    let children_ids = engine.get_children_str_ids_by_id(node_id).unwrap_or_default();
    if children_ids.is_empty() { return; }

    // compute geometry for the node
    for node_id in &children_ids {
        let (x, y, width, height) = compute_geometry(engine, parent_pos, node_id);
        
        // Store computed geometry in node properties for event system reuse
        // x, y are already absolute coordinates (computed_geometry includes parent_pos)
        engine.set_property_of_node(node_id, "computed_absolute_x", crate::AbstractValue::Number(x));
        engine.set_property_of_node(node_id, "computed_absolute_y", crate::AbstractValue::Number(y));
        engine.set_property_of_node(node_id, "computed_width", crate::AbstractValue::Number(width));
        engine.set_property_of_node(node_id, "computed_height", crate::AbstractValue::Number(height));
        
        let visible = engine.get_bool_property_of_node(node_id, "visible", true);
        if !visible {
            continue; // skip drawing if the node is not visible
        }
        
        let node_type = engine.get_node_type(node_id).unwrap_or(ItemTypeEnum::Node);
        // render the node according to its type
        match node_type {
            ItemTypeEnum::Rectangle => {
                let color = engine.get_color_property_of_node(node_id, "color", WHITE);
                draw_rectangle(x, y, width, height, color);
            }
            ItemTypeEnum::Text => {
                let text = engine.get_string_property_of_node(node_id, "text", String::new());
                let color = engine.get_color_property_of_node(node_id, "color", WHITE);
                let font_size = engine.get_number_property_of_node(node_id, "font_size", 20.0);

                let text_dimensions = measure_text(&text, None, font_size as u16, 1.0);
                engine.set_property_of_node(node_id, "implicit_width", text_dimensions.width.into());
                engine.set_property_of_node(node_id, "implicit_height", text_dimensions.height.into());
                
                let text_params = TextParams {
                    font_size: font_size as u16,
                    color,
                    ..Default::default()
                };

                draw_text_with_wrap(&text, x, y, width, &text_params);
            }
            ItemTypeEnum::MouseArea => {
                // MouseArea is invisible by default, but can have a debug color
                let debug = engine.get_bool_property_of_node(node_id, "debug", false);
                if debug {
                    let debug_color = engine.get_color_property_of_node(node_id, "debug_color", 
                        Color::from_rgba(255, 0, 0, 50)); // Semi-transparent red
                    draw_rectangle(x, y, width, height, debug_color);
                }
            }
            _ => {} // Node type and others
        }

        draw_childs(engine, node_id, (x, y));
    }
}
