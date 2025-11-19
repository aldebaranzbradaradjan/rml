// Example demonstrating component imports in RML

use macroquad::prelude::*;

use rml_core::prelude::*;
use rml_macros::rml;
use std::collections::HashMap;

fn window_conf() -> Conf {
    Conf {
        window_title: "RML Component Test".to_owned(),
        window_width: 600,
        window_height: 400,
        window_resizable: true,
        fullscreen: false,
        high_dpi: true,
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
        import "components" as UI

        Node {
            id: root
            anchors: fill
            color color: { DARKGRAY }

            // Use the imported Button component
            UI::Button {
                id: main_button
                anchors: top | left
                margins: 20
                text: "Click Me !"
                font: "liberation"
                on_click: {
                    println!("Main button clicked!");
                    $.info_card.content = "Main button clicked! (with unique ID) - see console output for more info about the event".to_string();
                }
            }

            UI::Button {
                anchors: top | right
                margins: 20
                text: "Click Me 2 !"
                font: "liberation"
                on_click: {
                    println!("Second button clicked!");
                    $.info_card.content = "Second button clicked! (with unique ID) - see console output for more info about the event".to_string();
                }
            }

            // Third button at bottom
            UI::ButtonRed {
                id: third_button
                anchors: bottom | left
                margins: 20
                text: { "Click Me" }
                font: "liberation"
                number count: 0
                on_click: {
                    println!("Third button clicked!");
                    $.info_card.content = "Third button clicked! (with unique ID) - see console output for more info about the event".to_string();
                    $.third_button.count += 1.;
                    update();
                }

                fn update() {
                    if $.third_button.count == 0.0 {
                        $.third_button.text = "Click Me".to_string();
                    }
                    else {
                        $.third_button.text = format!("Clicked : {}", $.third_button.count).to_string();
                    }
                }
            }

            // Use the imported Card component
            UI::Card {
                id: info_card
                anchors: center
                width: 400
                height: 150
                title: "Welcome"
                font: "liberation"
                string content: "Multiple buttons test - each should have unique IDs!"
            }
        }
    );

    println!("RML Component Test initialized");
    //println!("node from macro:\n {:#?}", engine.get_arena());

    let font = load_ttf_font("./LiberationSerif-Regular.ttf")
        .await
        .unwrap();
    engine.add_font("liberation".to_string(), font);

    loop {
        engine.process_events();
        clear_background(DARKGRAY);
        rml_core::draw::draw_root(&mut engine);
        next_frame().await
    }
}
