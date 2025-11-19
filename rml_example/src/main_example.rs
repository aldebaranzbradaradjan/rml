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
        // Importing components from the components folder with an alias access
        import "components" as UI

        Node {
            // Id, anchors, margins are special properties, internaly thez are string type
            id: root
            anchors: fill
            
            // Color properties expect a Color return type
            color color: { DARKGRAY } // color properties expect a Color return type

            // A simple counter stored in the root node it's a typed property
            number counter: 0

            // A custom signal that can be emitted, internally it's a boolean property
            signal clicked

            // The callback, block initializer and functions are all in plain Rust syntax
            // with the addition of the dollar syntax `$` to access nodes and properties
            // more easily, it is also possible to use the full API via the `engine` variable
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
                id: background_panel
                anchors: fill
                margins: 20
                color color: {
                    // color shifts slightly as counter increases
                    Color::new(0.52, 0.0, ($.root.counter/20.0).min(1.0), 1.0)
                }
            }

            // Text label
            Text {
                id: label
                anchors: center
                // Text property is typed as string
                string text: { format!("Counter: {}", $.root.counter) }
                // Font is not typed as string
                font: "liberation"

                // But they are string properties anyway.
                // It's important to type the properties if they will be used somewhere with the dollar sugar syntax
                // in other case, the engine will just use the property as needed without type checking.
                // Because internally, properties are stored as AbstractValue enum, and can be retyped at runtime.
                
                color color: { invert_color($.background_panel.color, 1.0) }
                number font_size: { compute_font_size() } // <= will generate a callback and an initialiser
                // if a property binding is detected in the callback, it is binded and re-evaluated when the property changes
            }

            // Button
            UI::Button {
                anchors: center | bottom
                margins: 30
                text: "Increment!"
                font: "liberation"

                on_click: {
                    // Emit a signal handled by the root node
                    emit!(engine, root, clicked);
                }
            }
        }
    );

    // We add a font to use in the RML
    let font = load_ttf_font("./LiberationSerif-Regular.ttf")
        .await
        .unwrap();
    engine.add_font("liberation".to_string(), font);

    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_root(&mut engine);
        next_frame().await
    }
}
