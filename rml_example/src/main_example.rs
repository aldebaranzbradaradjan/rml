// This example demonstrates how to use the RML library to create a simple 2D GUI
use rml_core::prelude::*;
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

    let mut engine = rml!(
        import "components" as UI

        Node {
            id: root
            anchors: fill
            color color: { DARKGRAY }

            // A simple counter stored in the root node it's a typed property
            number counter: 0

            // A custom signal that can be emitted, internally it's a boolean property
            signal clicked

            on_clicked: {
                // When the signal fires, increment the counter
                $.root.counter = $.root.counter + 1.0; // numbers are f32
                $.label.text = format!("Counter: {}", $.root.counter);
            }

            // A function used inside UI properties
            fn compute_font_size() -> u32 {
                // Dynamic but constant here—just to demonstrate usage
                18 + 6
            }

            // Background panel
            Rectangle {
                anchors: fill
                margins: 20
                color color: {
                    // color shifts slightly as counter increases
                    Color::new(0.5, ($.root.counter/20.0), 0.5, 1.0)
                }
            }

            // Text label
            Text {
                id: label
                anchors: center
                string text: { format!("Counter: {}", $.root.counter) }
                color color: { WHITE }
                number font_size: { compute_font_size() }
            }

            // Button
            UI::Button {
                anchors: center | bottom
                margins: 30
                text: "Increment!"

                on_click: {
                    // Emit a signal handled by the root node
                    emit!(engine, root, clicked);
                }
            }
        }
    );

    // println!("node from macro:\n {:#?}", engine.get_arena());

    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_root(&mut engine);
        next_frame().await
    }
}
