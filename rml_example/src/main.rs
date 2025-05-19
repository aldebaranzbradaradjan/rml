// This example demonstrates how to use the RML library to create a simple 2D GUI

use macroquad::prelude::*;

use std::collections::HashMap;
use rml_core::{ RmlEngine, Property, AbstractValue, get_number, set_number, get_string, set_string, ItemTypeEnum};
use rml_macros::rml;

fn window_conf() -> Conf {
    Conf {
        window_title: "RML Example".to_owned(),
        window_width: 500,
        window_height: 500,
        window_resizable: true,
        fullscreen: false,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandOnly,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    // Initialize the RML engine
    let mut engine = rml!(
        Node {
            id: root
            width: 500.0
            height: 500.0

            Rectangle {
                id: outer_rect
                x: {
                    let root_width = get_number!(engine, root, width);
                    let outer_rect_width = get_number!(engine, outer_rect, width);
                    let outer_rect_x = root_width / 2.0 - outer_rect_width / 2.0;
                    outer_rect_x
                }
                y: {
                    let root_height = get_number!(engine, root, height);
                    let outer_rect_height = get_number!(engine, outer_rect, height);
                    let outer_rect_y = root_height / 2.0 - outer_rect_height / 2.0;
                    outer_rect_y
                }
                width: 200
                height: 200
                color: "rgba(1.0, 0.0, 0.0, 1.0)"

                fn set_text() {
                    let x = engine.get_number_property_of_node("outer_rect", "x", 0.0);
                    let y = engine.get_number_property_of_node("outer_rect", "y", 0.0);
                    let text = format!("rml {x} {y}");
                    set_string!(engine, text, text, text);
                }

                on_x_changed: { set_text() }
                on_y_changed: { set_text() }
                
                Rectangle {
                    id: inner_rect
                    x: {
                        let outer_rect_width = get_number!(engine, outer_rect, width);
                        let inner_rect_width = get_number!(engine, inner_rect, width);
                        let inner_rect_x = outer_rect_width / 2.0 - inner_rect_width / 2.0;
                        inner_rect_x
                    }
                    y: {
                        let outer_rect_height = get_number!(engine, outer_rect, height);
                        let inner_rect_height = get_number!(engine, inner_rect, height);
                        let inner_rect_y = outer_rect_height / 2.0 - inner_rect_height / 2.0;
                        inner_rect_y
                    }
                    width: 160
                    height: 160
                    color: "rgba(0.3, 0.5, 0.3, 1.0)"

                    Rectangle {
                        id: inner_inner_rect
                        x: 150
                        y: 150
                        width: 10
                        height: 10
                        color: "rgba(0.0, 0., 0.3, 1.0)"
                    }

                    Node {
                        id: inner_inner_node
                        x: 0
                        y: 0
                        width: 100
                        height: 100

                        Text {
                            id: text
                            x: 0
                            y: 0
                            width: 100
                            height: 100
                            color: "rgba(0.0, 0.0, 1.0, 1.0)"
                            text: "rml"
                            font_size: 20
                        }
                    }
                }
            }
        }
    );

    // {
    //     // Demonstration of callbacks and bindings directly in rust code
    //     let cb_id = engine.add_callback(|engine| {
    //         let top_bar_x = engine.get_number_property_of_node("top_bar", "x", 0.0);
    //         let top_bar_width = engine.get_number_property_of_node("top_bar", "x", 0.0);
    //         engine.set_property_of_node("bottom_bar", "x", AbstractValue::Number(top_bar_x + (top_bar_width / 2.0)));
    //     });
    //     engine.bind_node_property_to_callback( "top_bar", "x", cb_id);
    // }

    println!("node from macro:\n {:#?}", engine.get_arena());

    // set macroquad window size


    loop {
        let mut x = engine.get_number_property_of_node("outer_rect", "x", 0.0);
        let mut y = engine.get_property_of_node("outer_rect", "y", 0.0, |v| v.to_number().map(|n| n as f32)); //ps.get_number_property_of_node(node.id.as_str(), "y", 0.0);

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

        engine.set_property_of_node("outer_rect", "x", AbstractValue::Number(x));
        engine.set_property_of_node("outer_rect", "y", AbstractValue::Number(y));
        
        // run callbacks if any property changed
        // this guarantees that the callbacks are run sequentially in the same frame and thread
        engine.run_callbacks();

        clear_background(BLACK);
        rml_core::draw::draw_childs(&engine, "root", 0., 0.);
        next_frame().await
    }
}
