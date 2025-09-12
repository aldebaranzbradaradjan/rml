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
        window_width: 300,
        window_height: 300,
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
        import "components" as UI

        Node {
            id: root
            anchors: fill
            text: "Please don't hit my button!"

            signal click

            on_click: {
                $.root.text = "outch!".to_string();
            }

            Rectangle {
                anchors: fill
                margins: 10
                color: { GRAY }
            }
            
            Text {
                anchors: center
                text: { $.root.text }
                color: { WHITE }
                font_size: 16
            }

            UI::Button {
                id: test_btn
                anchors: center | bottom
                margins: 20
                text: "Click me!"
                on_click: {
                    emit!(engine, root, click);
                }
            }
        }
    );

    println!("RML Component Test initialized");
    println!("node from macro:\n {:#?}", engine.get_arena());

    loop {
        engine.process_events();
        clear_background(DARKGRAY);
        rml_core::draw::draw_root(&mut engine);
        next_frame().await;
        info!("FPS: {}", get_fps());
    }
}
