use std::sync::Arc;

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

            Repeater {
                id: repeater_example

                number item_count: 3
                anchors: fill

                on_ready: {
                    let count = $.repeater_example.item_count as u32;
                    for i in 0 .. count {
                        repeater_example_create_item(&mut engine, i);
                    }
                }

                UI::ButtonRed {
                    number index: 0
                    text: { format!("Item {}", $.this.index) }
                    font: "liberation"
                    y: { $.this.index as f32 * 60.0 + 0.0 }
                    x: { $.this.index as f32 * 60.0 + 0.0 }
                    on_click: {
                        println!("Runtime generated button clicked!");
                        println!("Parent count: {}", $.parent.item_count);
                        println!("This button index: {}", $.this.index);
                    }
                }
            }
        }
    );

    let font = load_ttf_font("./LiberationSerif-Regular.ttf").await.unwrap();
    engine.add_font("liberation".to_string(), font);

    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_root(&mut engine);
        next_frame().await
    }

    // let mut engine = RmlEngine::new();
    // let new_id = Arc::new("test_node_1".to_string());

    // let n1 = new_id.clone();
    // let cb_id = engine.add_callback(move |engine| {
    //     let test = n1.clone();
    // });

    // let n2 = new_id.clone();
    // let cb_did = engine.add_callback(move |engine| {
    //     let testd = n2.clone();
    // });

}
