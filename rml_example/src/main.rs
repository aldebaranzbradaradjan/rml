// This example demonstrates how to use the RML library to create a simple 2D GUI

use macroquad::prelude::*;

use std::collections::HashMap;
use rml_core::{RmlEngine, Property, AbstractValue};
use rml_macros::rml;

#[macroquad::main("Simple RML Example")]
async fn main() {

    // let test = Property::new(AbstractValue::String("test".to_string()) );

    let mut engine = rml!(
        Node {
            id: root
            width: 300.0
            height: 300.0

            Rectangle {
                id: top_bar
                x: 0.0
                y: 0.0
                width: 300
                height: 10.0
                color: "rgba(1.0, 0.0, 0.0, 1.0)"

                on_x_change: {
                    let top_bar_x = engine.get_number_property_of_node("top_bar", "x", 0.0);
                    let top_bar_width = engine.get_number_property_of_node("top_bar", "width", 0.0);
                    let bottom_bar_width = engine.get_number_property_of_node("bottom_bar", "width", 0.0);
                    engine.set_property_of_node("bottom_bar", "x", AbstractValue::Number(top_bar_x + (top_bar_width / 2.0) - (bottom_bar_width / 2.0)));
                }
                
                on_y_change: {
                    let val = engine.get_number_property_of_node("top_bar", "y", 0.0);
                    engine.set_property_of_node("bottom_bar", "y", AbstractValue::Number(val + 20.0));
                }
            }

            Rectangle {
                id: bottom_bar
                x: 0.0
                y: 20.0
                width: 25
                height: 25
                color: "rgba(1.0, 0.0, 1.0, 1.0)"
            }
        }
    );


    // {
    //     // Demonstration of callbacks and bindings
    //     let cb_id = engine.add_callback(|engine| {
    //         let top_bar_x = engine.get_number_property_of_node("top_bar", "x", 0.0);
    //         let top_bar_width = engine.get_number_property_of_node("top_bar", "x", 0.0);
    //         engine.set_property_of_node("bottom_bar", "x", AbstractValue::Number(top_bar_x + (top_bar_width / 2.0)));
    //     });

    //     engine.bind_node_property_to_callback( "top_bar", "x", cb_id);
    // }

    println!("node from macro:\n {:#?}", engine.get_arena());

    loop {
        let mut x = engine.get_number_property_of_node("top_bar", "x", 0.0);
        let mut y = engine.get_property_of_node("top_bar", "y", 0.0, |v| v.to_number().map(|n| n as f32)); //ps.get_number_property_of_node(node.id.as_str(), "y", 0.0);

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
        
        // run callbacks if any property changed
        // this guarantees that the callbacks are run sequentially in the same frame and thread
        engine.run_callbacks();

        clear_background(WHITE);
        rml_core::draw::draw_element(&engine, "root");
        next_frame().await
    }
}
