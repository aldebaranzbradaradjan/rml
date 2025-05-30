// This example demonstrates how to use the RML library to create a simple 2D GUI

use macroquad::prelude::*;

use std::collections::HashMap;
use rml_core::{ RmlEngine, Property, AbstractValue, get_number, set_number, get_string, get_key_event, SystemEvent, EventType, set_string, ItemTypeEnum};
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

            // a rectangle anchored to fill the root node
            Rectangle {
                width: 10
                height: 10
                anchors: fill
                color: "rgba(1.0, 0.0, 0.0, 0.3)"
            }
            
            // a rectangle anchored to fill the root node, but with margins
            Rectangle {
                width: 10
                height: 10
                anchors: fill
                margins: 50
                color: "rgba(0.0, 1.0, 0.0, 0.3)"
            }

            // a rectangle anchored in top left corner of the root node
            Rectangle {
                width: 10
                height: 10
                anchors: top | left
                color: "rgba(0.0, 1.0, 0.0, 1.0)"
            }

            // a rectangle anchored in top right corner of the root node
            Rectangle {
                width: 10
                height: 10
                anchors: top | right
                color: "rgba(0.0, 0.0, 1.0, 1.0)"
            }

            // a rectangle anchored in bottom left corner of the root node
            Rectangle {
                width: 10
                height: 10
                anchors: bottom | left
                color: "rgba(1.0, 1.0, 0.0, 1.0)"
            }

            // a rectangle anchored in bottom right corner of the root node
            Rectangle {
                width: 10
                height: 10
                anchors: bottom | right
                color: "rgba(1.0, 0.0, 1.0, 1.0)"
            }

            // a rectangle anchored at left and right of the root node and centered vertically
            Rectangle {
                width: 10
                height: 10
                anchors: left | right | vertical_center
                color: "rgba(0.5, 0.0, 0.5, 1.0)"
            }

            // a rectangle anchored at top and bottom of the root node and centered horizontally
            Rectangle {
                width: 10
                height: 10
                anchors: top | bottom | horizontal_center
                color: "rgba(0.5, 0.5, 0.0, 1.0)"
            }

            // a rectangle anchored in the center of the root node
            Rectangle {
                width: 10
                height: 10
                anchors: center
                color: "rgba(0.5, 0.7, 0.5, 1.0)"
            }

            // a text node anchored to the top left corner of the root node with a margin and a width specified
            Text {
                anchors: top | left
                margins: 20
                width: 30
                color: "rgba(1.0, 1.0, 1.0, 1.0)"
                text: "Test Text"
                font_size: 20
            }

            Text {
                x: 20
                y: 0
                color: "rgba(1.0, 1.0, 1.0, 1.0)"
                text: "Test Text"
                font_size: 20
            }

            // a smiling face made of rectangles
            Rectangle {
                anchors: right | top
                margins: 20
                width: 100
                height: 100
                color: "rgba(1.0, 1.0, 1.0, 1.0)"
                
                Rectangle {
                    anchors: left | top
                    margins: 30
                    width: 10
                    height: 10
                    color: "rgba(0., 0., 0., 1.0)"
                }
                Rectangle {
                    anchors: right | top
                    margins: 30
                    width: 10
                    height: 10
                    color: "rgba(0., 0., 0., 1.0)"
                }
                Rectangle {
                    anchors: bottom | horizontal_center
                    bottom_margin: 30
                    width: 70
                    height: 10
                    color: "rgba(0., 0., 0., 1.0)"

                    Rectangle {
                        anchors: left | top
                        left_margin: -10
                        top_margin: -10
                        width: 10
                        height: 10
                        color: "rgba(0., 0., 0., 1.0)"
                    }
                    Rectangle {
                        anchors: right | top
                        right_margin: -10
                        top_margin: -10
                        width: 10
                        height: 10
                        color: "rgba(0., 0., 0., 1.0)"
                    }
                }
            }

            Rectangle {
                id: outer_rect
                x: {
                    engine.set_focused_node("outer_rect");
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

                on_key_down: {
                    let mut x = get_number!(engine, outer_rect, x);
                    let mut y = get_number!(engine, outer_rect, y);

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

                    set_number!(engine, outer_rect, x, x);
                    set_number!(engine, outer_rect, y, y);
                }

                width: 200
                height: 200
                color: "rgba(1.0, 1.0, 1.0, 1.0)"

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
                        outer_rect_width / 2.0 - inner_rect_width / 2.0
                    }
                    y: {
                        let outer_rect_height = get_number!(engine, outer_rect, height);
                        let inner_rect_height = get_number!(engine, inner_rect, height);
                        outer_rect_height / 2.0 - inner_rect_height / 2.0
                    }
                    
                    width: 160
                    height: 160
                    color: "rgba(0.3, 0.3, 0.3, 1.0)"

                    Rectangle {
                        id: left_eye
                        x: 30
                        y: 30
                        width: 10
                        height: 10
                        color: "rgba(0., 0., 0., 1.0)"
                    }

                    Rectangle {
                        id: right_eye
                        x: 120
                        y: 30
                        width: 10
                        height: 10
                        color: "rgba(0., 0., 0., 1.0)"
                    }

                    Rectangle {
                        id: mouth

                        // margins
                        margins: 2
                        bottom_margin: 50
                        anchors: center

                        // left_margin: 0
                        // right_margin: 0
                        // top_margin: 0
                        // bottom_margin: 0

                        // anchors, if used in a non layouted node, it place the node relative to the parent node
                        // anchors: top | bottom | left | right | horizontal_center | vertical_center | center | fill
                        // if used in a layouted node, it place the node relative to a box in the layout ?
                        // not sure is this make sense, maybe just limit the anchors to the non layouted node
                        // under the hood, the property is a string, so it can be used as a string
                        // anchors: horizontal_center | vertical_center | left | right | top | bottom 

                        // // layout for child of layouted node
                        // // under the hood, the property name is like this : "layout_left_margin"
                        layout.fill_width: true
                        // layout.fill_height: true
                        // layout.min_width: 10
                        // layout.min_height: 10
                        // layout.max_width: 10
                        // layout.max_height: 10

                        // // // layout for layouted node
                        // layout.mode: column
                        // layout.mode: row
                        // layout.mode: grid
                        // layout.spacing: 10
                        // layout.vertical_spacing: 10
                        // layout.horizontal_spacing: 10


                        x: 30
                        y: 100
                        width: 100
                        height: 10
                        color: "rgba(0., 0., 0., 1.0)"

                        Rectangle {
                            id: mouth_l
                            x: -10
                            y: -10
                            width: 10
                            height: 10
                            color: "rgba(0., 0., 0., 1.0)"
                        }

                        Rectangle {
                            id: mouth_r
                            x: 100
                            y: -10
                            width: 10
                            height: 10
                            color: "rgba(0., 0., 0., 1.0)"
                        }
                    }

                    Node {
                        id: inner_inner_node
                        x: 10
                        y: 0
                        width: 100
                        height: 100

                        Text {
                            id: text
                            x: 0
                            y: 5
                            width: 100
                            height: 100
                            color: "rgba(1.0, 1.0, 1.0, 1.0)"
                            text: "rml"
                            font_size: 20
                        }
                    }

                    Rectangle {
                        anchors: left | right | bottom
                        bottom_margin: 10
                        width: 10
                        height: 10
                        color: "rgba(1., 0., 0., 1.0)"
                    }
                }
            }
        

        }
    );

    println!("node from macro:\n {:#?}", engine.get_arena());

    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await
    }
}
