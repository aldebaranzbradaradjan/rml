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
        window_title: "RML Smiley Demo 😊".to_owned(),
        window_width: 800,
        window_height: 600,
        window_resizable: true,
        fullscreen: false,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandOnly,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn create_smiley_ui() -> RmlEngine {
    rml! {
        import "components" as UI
        
        Node {
            id: root
            anchors: fill
            color: { GRAY }

            on_ready: {
                emit!( engine, sad_btn, click ); 
            }
            
            // Title
            Text {
                id: title
                anchors: horizontal_center | top
                top_margin: 50
                text: "Smiley Mood Controller :)"
                color: { WHITE }
                font_size: 24
            }

            Node {
                width: { $.root.width:f32 / 2.0 }
                height: { $.root.height }
                anchors: left

                UI::Button {
                    id: sad_btn
                    anchors: center
                    text: "SAD"
                    on_click: {
                        let value: AbstractValue = { lighter_color(GREEN, 0.5) }.into();
                        engine.set_property_of_node("face", "color", value);

                        $.left_cheek.left_margin = 40.;
                        $.left_cheek.top_margin = 85.;
                        $.left_cheek.width = 10.;
                        $.left_corner.top_margin = 5.;

                        let value: AbstractValue = { lighter_color(BLUE, 3.) }.into();
                        engine.set_property_of_node("left_cheek", "color", value);
                    }
                }
            }

            Node {
                width: { $.root.width:f32 / 2.0 }
                height: { $.root.height }
                anchors: right

                UI::Button {
                    id: happy_btn
                    anchors: center
                    text: "HAPPY"
                    on_click: {
                        let value: AbstractValue = { lighter_color(YELLOW, 0.1) }.into();
                        engine.set_property_of_node("face", "color", value);

                        $.left_cheek.left_margin = 20.;
                        $.left_cheek.top_margin = 105.;
                        $.left_cheek.width = 20.;
                        $.left_corner.top_margin = -5.;

                        let value: AbstractValue = { lighter_color(RED, 0.5) }.into();
                        engine.set_property_of_node("left_cheek", "color", value);
                    }
                }
            }
            
            // Smiley face
            Rectangle {
                id: face
                anchors: center
                width: 200
                height: 200
                color: { YELLOW }
                
                // Left eye
                Rectangle {
                    id: left_eye
                    anchors: left | top
                    left_margin: 50
                    top_margin: 63
                    width: 25
                    height: { $.left_eye.width }
                    color: { BLACK }
                }
                
                // Right eye
                Rectangle {
                    id: right_eye
                    anchors: right | top
                    right_margin: { $.left_eye.left_margin }
                    top_margin: { $.left_eye.top_margin }
                    width: { $.left_eye.width }
                    height: { $.left_eye.width }
                    color: { BLACK }
                }

                Rectangle {
                    id: left_cheek
                    anchors: left | top
                    left_margin: 40
                    top_margin: 85
                    width: 10
                    height: { $.left_cheek.width }
                    color: { lighter_color(RED, 0.5) }
                }
                
                // Right eye
                Rectangle {
                    id: right_cheek
                    anchors: right | top
                    right_margin: { $.left_cheek.left_margin }
                    top_margin: { $.left_cheek.top_margin }
                    width: { $.left_cheek.width }
                    height: { $.left_cheek.width }
                    color: { $.left_cheek.color }
                }
                
                // Mouth (starts happy)
                Rectangle {
                    id: mouth
                    anchors: center | bottom
                    bottom_margin: 40
                    width: 100
                    height: 10
                    color: { BLACK }

                    // corner
                    Rectangle {
                        id: left_corner
                        anchors: top | left
                        top_margin: -10
                        left_margin: -5
                        width: { $.mouth.height }
                        height: { $.mouth.height }
                        color: { BLACK }
                    }

                    // corner
                    Rectangle {
                        id: right_corner
                        anchors: top | right
                        top_margin: { $.left_corner.top_margin }
                        right_margin: { $.left_corner.left_margin }
                        width: { $.mouth.height }
                        height: { $.mouth.height }
                        color: { BLACK }
                    }
                }
            }
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut engine = create_smiley_ui();

    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_root(&mut engine);
        next_frame().await;
    }
}