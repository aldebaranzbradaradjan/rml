// This example demonstrates how to use the RML library to create a simple 2D GUI

use macroquad::prelude::*;

use std::collections::HashMap;
use rml_core::{ RmlEngine, Property, AbstractValue, get_number, set_number, ItemTypeEnum};
use rml_macros::rml;

fn window_conf() -> Conf {
    Conf {
        window_title: "Window Conf".to_owned(),
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
    // let mut engine = rml!(
    //     Node {
    //         id: root
    //         width: 500.0
    //         height: 500.0

    //         Rectangle {
    //             id: top_bar
    //             x: { // initilizers are executed at runtime after the node is created
    //                 // center the top bar
    //                 let root_width = get_number!(engine, root, width);
    //                 let top_bar_width = get_number!(engine, top_bar, width);
    //                 let top_bar_x = root_width / 2.0 - top_bar_width / 2.0;
    //                 top_bar_x
    //             }
    //             y: {
    //                 let root_height = get_number!(engine, root, height);
    //                 let top_bar_height = get_number!(engine, top_bar, height);
    //                 let top_bar_y = root_height / 2.0 - top_bar_height / 2.0;
    //                 top_bar_y
    //             }
    //             width: 200
    //             height: 200.0
    //             color: "rgba(1.0, 0.0, 0.0, 1.0)"
                
    //             // functions are defined at the end of the node generation code
    //             fn compute_bottom_bar_x() -> f32 {
    //                 let top_bar_x = get_number!(engine, top_bar, x);
    //                 let top_bar_width = get_number!(engine, top_bar, width);
    //                 let bottom_bar_width = get_number!(engine, bottom_bar, width);
    //                 let bottom_bar_x = top_bar_x + top_bar_width / 2.0 - bottom_bar_width / 2.0;
    //                 return bottom_bar_x;
    //             }

    //             on_x_changed: { // this is a callback that is executed when the x property of the top_bar changes
    //                 let x = compute_bottom_bar_x();
    //                 set_number!(engine, bottom_bar, x, x);
    //             }
                
    //             on_y_changed: {
    //                 let val = engine.get_number_property_of_node("top_bar", "y", 0.0);
    //                 engine.set_property_of_node("bottom_bar", "y", AbstractValue::Number(val + 20.0));
    //             }

    //             Rectangle {
    //                 id: inner_rect
    //                 x: 20.0
    //                 y: 20.0
    //                 width: 25
    //                 height: 25
    //                 color: "rgba(0.3, 0.5, 0.3, 1.0)"
    //             }
    //         }

    //         Rectangle {
    //             id: bottom_bar
    //             x: { compute_bottom_bar_x() }
    //             y: 20.0
    //             width: 250
    //             height: 25
    //             color: "rgba(1.0, 0.0, 1.0, 1.0)"
    //         }
    //     }
    // );

    let mut engine = rml!(
        Node {
            id: root
            width: 500.0
            height: 500.0

            // draw a rectangle at the center of the screen
            // In this rectangle, draw 4 Rectangles to create border of 10 px,
            // then draw rectangle in the right bottom edge to draw a simplifier RML letters
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
                        x: 30
                        y: 30
                        width: 100
                        height: 100
                        color: "rgba(0.0, 0.5, 0.3, 1.0)"
                    }

                    Node {
                        id: inner_inner_node
                        x: 35
                        y: 35
                        width: 100
                        height: 100
                        color: "rgba(0.0, 0.5, 0.3, 1.0)"

                        Text {
                            id: inner_inner_text
                            x: 0
                            y: 0
                            width: 100
                            height: 100
                            color: "rgba(1.0, 1.0, 1.0, 1.0)"
                            text: "rml"
                            font_size: 30
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
