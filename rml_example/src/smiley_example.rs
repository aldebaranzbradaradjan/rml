use macroquad::prelude::*;

use rml_core::prelude::*;
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
            color color: { GRAY }

            on_ready: {
                emit!( engine, sad_btn, click ); 
            }
            
            // Title
            Text {
                id: title
                anchors: horizontal_center | top
                number top_margin: 50
                string text: "Smiley Mood Controller :)"
                color color: { WHITE }
                number font_size: 24
            }

            Node {
                number width: { $.root.width / 2.0 }
                number height: { $.root.height }
                anchors: left

                UI::Button {
                    id: sad_btn
                    anchors: center
                    text: "SAD"
                    on_click: {
                        $.face.color = lighter_color(GREEN, 0.5);
                        $.left_cheek.left_margin = 40.;
                        $.left_cheek.top_margin = 85.;
                        $.left_cheek.width = 10.;
                        $.left_corner.top_margin = 5.;
                        $.left_cheek.color = lighter_color(BLUE, 0.8);
                    }
                }
            }

            Node {
                number width: { $.root.width / 2.0 }
                number height: { $.root.height }
                anchors: right

                UI::Button {
                    id: happy_btn
                    anchors: center
                    text: "HAPPY"
                    on_click: {
                        $.face.color = lighter_color(YELLOW, 0.1);
                        $.left_cheek.left_margin = 20.;
                        $.left_cheek.top_margin = 105.;
                        $.left_cheek.width = 20.;
                        $.left_corner.top_margin = -5.;
                        $.left_cheek.color = lighter_color(RED, 0.5);
                    }
                }
            }
            
            // Smiley face
            Rectangle {
                id: face
                anchors: center
                number width: 200
                number height: 200
                color color: { YELLOW }
                
                // Left eye
                Rectangle {
                    id: left_eye
                    anchors: left | top
                    number left_margin: 50
                    number top_margin: 63
                    number width: 25
                    number height: { $.left_eye.width }
                    color color: { BLACK }
                }
                
                // Right eye
                Rectangle {
                    id: right_eye
                    anchors: right | top
                    number right_margin: { $.left_eye.left_margin }
                    number top_margin: { $.left_eye.top_margin }
                    number width: { $.left_eye.width }
                    number height: { $.left_eye.width }
                    color color: { BLACK }
                }

                Rectangle {
                    id: left_cheek
                    anchors: left | top
                    number left_margin: 40
                    number top_margin: 85
                    number width: 10
                    number height: { $.left_cheek.width }
                    color color: { lighter_color(RED, 0.5) }
                }
                
                // Right eye
                Rectangle {
                    id: right_cheek
                    anchors: right | top
                    number right_margin: { $.left_cheek.left_margin }
                    number top_margin: { $.left_cheek.top_margin }
                    number width: { $.left_cheek.width }
                    number height: { $.left_cheek.width }
                    color color: { $.left_cheek.color }
                }
                
                // Mouth (starts happy)
                Rectangle {
                    id: mouth
                    anchors: center | bottom
                    number bottom_margin: 40
                    number width: 100
                    number height: 10
                    color color: { BLACK }

                    // corner
                    Rectangle {
                        id: left_corner
                        anchors: top | left
                        number top_margin: -10
                        number left_margin: -5
                        number width: { $.mouth.height }
                        number height: { $.mouth.height }
                        color color: { BLACK }
                    }

                    // corner
                    Rectangle {
                        id: right_corner
                        anchors: top | right
                        number top_margin: { $.left_corner.top_margin }
                        number right_margin: { $.left_corner.left_margin }
                        number width: { $.mouth.height }
                        number height: { $.mouth.height }
                        color color: { BLACK }
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