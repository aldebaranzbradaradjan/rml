// Example demonstrating component imports in RML

use macroquad::prelude::*;

use rml_core::{
    darker_color, decompose_color_string, emit, get_bool, get_color, get_key_event, get_number,
    get_string, get_value, lighter_color, set_bool, set_number, set_string, AbstractValue,
    EventType, ItemTypeEnum, Property, RmlEngine, SystemEvent,
};
use rml_macros::rml;
use std::collections::HashMap;

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
                text: { "Click Me" }
                count: 0
                on_click: {
                    println!("Third button clicked!");
                    $.info_card.content = "Third button clicked! (with unique ID) - see console output for more info about the event".to_string();
                    $.third_button.count += 1.;
                    /*
                    same as : $.third_button.count = $.third_button.count:f32 + 1.;
                    note the :f32 to indicate to the macro that we need a f32 conversion
                    same as $.third_button.count = $.third_button.count.to_number().unwrap() + 1.;
                    in some case, the contexte is sufficient to infer the type
                    */
                    //$.third_button.text = format!("Clicked : {}", $.third_button.count).to_string();
                    update();
                }

                fn update() {
                    let count = $.third_button.count.to_number().unwrap() as i32;
                    let count = $.third_button.count:f32 as i32;
                    if count == 0 {
                        $.third_button.text = "Click Me".to_string();
                    }
                    else {
                        $.third_button.text = format!("Clicked : {}", $.third_button.count).to_string();
                    }
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
