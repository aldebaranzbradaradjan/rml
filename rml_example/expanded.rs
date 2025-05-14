#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use macroquad::prelude::*;
use std::collections::HashMap;
use rml_core::{RmlEngine, Property, AbstractValue, get_number, set_number, ItemTypeEnum};
use rml_macros::rml;
fn main() {
    macroquad::Window::new("Simple RML Example", amain());
}
async fn amain() {
    let mut engine = {
        let mut engine = RmlEngine::new();
        let temp_node_root = engine
            .add_node("root".to_string(), ItemTypeEnum::Node, HashMap::new())
            .unwrap();
        let prop_id = engine
            .add_property(Property::new(AbstractValue::String("root".to_string())));
        engine.add_property_to_node(temp_node_root, "id".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(300f32)));
        engine.add_property_to_node(temp_node_root, "width".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(300f32)));
        engine.add_property_to_node(temp_node_root, "height".to_string(), prop_id);
        let temp_node_top_bar = engine
            .add_node("top_bar".to_string(), ItemTypeEnum::Rectangle, HashMap::new())
            .unwrap();
        let prop_id = engine
            .add_property(Property::new(AbstractValue::String("top_bar".to_string())));
        engine.add_property_to_node(temp_node_top_bar, "id".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(0f32)));
        engine.add_property_to_node(temp_node_top_bar, "x".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(0f32)));
        engine.add_property_to_node(temp_node_top_bar, "y".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(300f32)));
        engine.add_property_to_node(temp_node_top_bar, "width".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(10f32)));
        engine.add_property_to_node(temp_node_top_bar, "height".to_string(), prop_id);
        let prop_id = engine
            .add_property(
                Property::new(
                    AbstractValue::String("rgba(1.0, 0.0, 0.0, 1.0)".to_string()),
                ),
            );
        engine.add_property_to_node(temp_node_top_bar, "color".to_string(), prop_id);
        let cb_id = engine
            .add_callback(|engine| {
                compute_bottom_bar_x(engine);
            });
        engine.bind_node_property_to_callback("top_bar", "x", cb_id);
        let cb_id = engine
            .add_callback(|engine| {
                let val = engine.get_number_property_of_node("top_bar", "y", 0.0);
                engine
                    .set_property_of_node(
                        "bottom_bar",
                        "y",
                        AbstractValue::Number(val + 20.0),
                    );
            });
        engine.bind_node_property_to_callback("top_bar", "y", cb_id);
        let temp_node_inner_rect = engine
            .add_node("inner_rect".to_string(), ItemTypeEnum::Rectangle, HashMap::new())
            .unwrap();
        let prop_id = engine
            .add_property(
                Property::new(AbstractValue::String("inner_rect".to_string())),
            );
        engine.add_property_to_node(temp_node_inner_rect, "id".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(20f32)));
        engine.add_property_to_node(temp_node_inner_rect, "x".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(20f32)));
        engine.add_property_to_node(temp_node_inner_rect, "y".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(25f32)));
        engine.add_property_to_node(temp_node_inner_rect, "width".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(25f32)));
        engine.add_property_to_node(temp_node_inner_rect, "height".to_string(), prop_id);
        let prop_id = engine
            .add_property(
                Property::new(
                    AbstractValue::String("rgba(0.3, 0.5, 0.3, 1.0)".to_string()),
                ),
            );
        engine.add_property_to_node(temp_node_inner_rect, "color".to_string(), prop_id);
        engine.add_child(temp_node_top_bar, temp_node_inner_rect);
        engine.add_child(temp_node_root, temp_node_top_bar);
        let temp_node_bottom_bar = engine
            .add_node("bottom_bar".to_string(), ItemTypeEnum::Rectangle, HashMap::new())
            .unwrap();
        let prop_id = engine
            .add_property(
                Property::new(AbstractValue::String("bottom_bar".to_string())),
            );
        engine.add_property_to_node(temp_node_bottom_bar, "id".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(0f32)));
        engine.add_property_to_node(temp_node_bottom_bar, "x".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(20f32)));
        engine.add_property_to_node(temp_node_bottom_bar, "y".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(25f32)));
        engine.add_property_to_node(temp_node_bottom_bar, "width".to_string(), prop_id);
        let prop_id = engine.add_property(Property::new(AbstractValue::Number(25f32)));
        engine.add_property_to_node(temp_node_bottom_bar, "height".to_string(), prop_id);
        let prop_id = engine
            .add_property(
                Property::new(
                    AbstractValue::String("rgba(1.0, 0.0, 1.0, 1.0)".to_string()),
                ),
            );
        engine.add_property_to_node(temp_node_bottom_bar, "color".to_string(), prop_id);
        engine.add_child(temp_node_root, temp_node_bottom_bar);
        fn compute_bottom_bar_x(engine: &mut RmlEngine) {
            let top_bar_x = { engine.get_number_property_of_node("top_bar", "x", 0.0) };
            let top_bar_width = {
                engine.get_number_property_of_node("top_bar", "width", 0.0)
            };
            let bottom_bar_width = {
                engine.get_number_property_of_node("bottom_bar", "width", 0.0)
            };
            let bottom_bar_x = top_bar_x + top_bar_width / 2.0 - bottom_bar_width / 2.0;
            {
                engine
                    .set_property_of_node(
                        "bottom_bar",
                        "x",
                        AbstractValue::Number(bottom_bar_x),
                    )
            };
        }
        engine
    };
    {
        ::std::io::_print(
            format_args!("node from macro:\n {0:#?}\n", engine.get_arena()),
        );
    };
    loop {
        let mut x = engine.get_number_property_of_node("top_bar", "x", 0.0);
        let mut y = engine
            .get_property_of_node(
                "top_bar",
                "y",
                0.0,
                |v| v.to_number().map(|n| n as f32),
            );
        if is_key_down(KeyCode::Right) {
            x += 1.0;
        }
        if is_key_down(KeyCode::Left) {
            x -= 1.0;
        }
        if is_key_down(KeyCode::Down) {
            y += 1.0;
        }
        if is_key_down(KeyCode::Up) {
            y -= 1.0;
        }
        engine.set_property_of_node("top_bar", "x", AbstractValue::Number(x));
        engine.set_property_of_node("top_bar", "y", AbstractValue::Number(y));
        engine.run_callbacks();
        clear_background(WHITE);
        rml_core::draw::draw_element(&engine, "root");
        next_frame().await
    }
}
