// Example demonstrating component imports in RML

use macroquad::prelude::*;

use std::collections::HashMap;
use rml_core::{ RmlEngine, Property, AbstractValue, get_number, set_number, get_string, get_key_event, SystemEvent, EventType, set_string, set_bool, ItemTypeEnum};
use rml_macros::rml;

fn window_conf() -> Conf {
    Conf {
        window_title: "RML Component Test".to_owned(),
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
    // Initialize the RML engine with imported components
    let mut engine = rml!(
        import "components"
        
        Node {
            id: root
            width: 600.0
            height: 400.0
            
            // Use the imported Button component
            Button {
                id: main_button
                anchors: top | left
                margins: 20
                text: "Click Me!"
                on_clicked_changed: {
                    println!("Main button clicked!");
                    set_string!(engine, info_card, content, "Main button clicked! (with unique ID) - see console output for more info about the event".to_string());
                }
            }

            // Second button to test unique IDs
            Button {
                id: second_button
                anchors: top | right
                margins: 20
                text: "Button 2"
                on_clicked_changed: {
                    println!("Second button clicked!");
                    set_string!(engine, info_card, content, "Second button clicked! (with unique ID) - see console output for more info about the event".to_string());
                }
            }
            
            // Third button at bottom
            Button {
                //id: third_button
                anchors: bottom | left
                margins: 20
                text: "Button 3"
                color: "rgba(0.8, 0.2, 0.2, 1.0)"
                on_clicked_changed: {
                    println!("Third button clicked!");
                    //set_bool!(engine, id7e5fd640c9f94a9fb80210410ab5a6f9, visible, true);
                    set_string!(engine, info_card, content, "Third button clicked! (with unique ID) - see console output for more info about the event".to_string());
                }
            }
            
            //Use the imported Card component
            Card {
                id: info_card
                anchors: center
                title: "Welcome"
                content: "Multiple buttons test - each should have unique IDs!"
            }
        }
    );

    println!("RML Component Test initialized");
    println!("node from macro:\n {:#?}", engine.get_arena());

    loop {
        engine.process_events();
        clear_background(DARKGRAY);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await
    }
}
