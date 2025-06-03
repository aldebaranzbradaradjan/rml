// Example demonstrating dollar ($) syntax in RML

use macroquad::prelude::*;

use std::collections::HashMap;
use rml_core::{ RmlEngine, Property, AbstractValue, get_value, get_number, set_number, get_string, get_key_event, SystemEvent, EventType, set_string, set_bool, ItemTypeEnum};
use rml_macros::rml;

fn window_conf() -> Conf {
    Conf {
        window_title: "RML Dollar Syntax Test".to_owned(),
        window_width: 600,
        window_height: 400,
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
    // Initialize the RML engine with dollar syntax
    let mut engine = rml!(
        Node {
            id: root
            width: 600.0
            height: 400.0
            
            Rectangle {
                id: counter_display
                anchors: center
                width: 300
                height: 100
                color: "rgba(0.9, 0.9, 0.9, 1.0)"
                counter: 0
                
                
                Text {
                    id: counter_text
                    anchors: center
                    text: { $.counter_display.counter }
                    color: "rgba(0.0, 0.0, 0.0, 1.0)"
                    font_size: 24
                }
            }
            
            Rectangle {
                id: increment_btn
                anchors: bottom | left
                margins: 50
                width: 120
                height: 40
                color: "rgba(0.3, 0.8, 0.6, 1.0)"
                
                Text {
                    anchors: center
                    text: "Increment"
                    color: "rgba(1.0, 1.0, 1.0, 1.0)"
                    font_size: 14
                }
                
                MouseArea {
                    anchors: fill
                    
                    on_click: {
                        // Test dollar syntax: increment counter
                        $.counter_display.counter += 1.;
                        println!("Counter incremented to: {}", $.counter_display.counter);
                    }
                }
            }
            
            Rectangle {
                id: decrement_btn
                anchors: bottom | right
                margins: 50
                width: 120
                height: 40
                color: { Color::new(0.8, 0.3, 0.3, 1.0) }
                
                Text {
                    anchors: center
                    text: "Decrement"
                    color: { WHITE }
                    font_size: 14
                }
                
                MouseArea {
                    anchors: fill
                    
                    on_click: {
                        $.counter_display.counter -= 1.;
                        println!("Counter decremented to: {}", $.counter_display.counter);
                    }
                }
            }
            
            Rectangle {
                id: reset_btn
                anchors: bottom | horizontal_center
                margins: 50
                width: 100
                height: 40
                color: "rgba(0.5, 0.5, 0.5, 1.0)"
                
                Text {
                    anchors: center
                    text: "Reset"
                    color: "rgba(1.0, 1.0, 1.0, 1.0)"
                    font_size: 14
                }
                
                MouseArea {
                    anchors: fill
                    
                    on_click: {
                        // Test dollar syntax: reset counter
                        $.counter_display.counter = 0.;
                        println!("Counter reset to: {}", $.counter_display.counter);
                    }
                }
            }
            
            Text {
                id: instructions
                anchors: top | horizontal_center
                margins: 20
                text: "Click buttons to test $ syntax - check console for debug output"
                color: "rgba(0.2, 0.2, 0.2, 1.0)"
                font_size: 12
            }
        }
    );

    println!("RML Dollar Syntax Test initialized");
    println!("Use $.node_id.property syntax for cleaner property access");

    loop {
        engine.process_events();
        clear_background(LIGHTGRAY);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await
    }
}