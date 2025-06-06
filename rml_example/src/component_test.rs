// Example demonstrating component imports in RML

use macroquad::prelude::*;

use std::collections::HashMap;
use rml_core::{ RmlEngine, Property, AbstractValue, emit, darker_color, lighter_color, get_color, get_value, get_bool, set_bool, decompose_color_string, get_number, set_number, get_string, get_key_event, SystemEvent, EventType, set_string, ItemTypeEnum};
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
                on_click: {
                    println!("Main button clicked!");
                    $.info_card.content = "Main button clicked! (with unique ID) - see console output for more info about the event".to_string();
                }
            }

            // Second button to test unique IDs
            Button {
                anchors: top | right
                margins: 20
                text: "Button 2"
                on_click: {
                    println!("Second button clicked!");
                    $.info_card.content = "Second button clicked! (with unique ID) - see console output for more info about the event".to_string();
                }
            }
            
            // Third button at bottom
            ButtonRed {
                id: third_button
                anchors: bottom | left
                margins: 20
                text: "Button 3"
                count: 0
                on_click: {
                    println!("Third button clicked!");
                    $.info_card.content = "Third button clicked! (with unique ID) - see console output for more info about the event".to_string();
                    $.third_button.count += 1.;
                    /* 
                    same as : $.third_button.count = $.third_button.count:f32 + 1.; 
                    note the :f32 to indicate to the macro that we need a f32 conversion
                    same as $.third_button.count = $.third_button.count.to_number().unwrap() + 1.;
                    */
                    
                    // in some case, the contexte is sufficient to infer the type
                    $.third_button.text = format!("Clicked : {}", $.third_button.count).to_string();
                }
            }
            
            // Use the imported Card component
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
