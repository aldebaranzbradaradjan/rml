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

            UI::Button {
                anchors: center | bottom
                margins: 50
                text: "Increment!"
                font: "liberation"

                on_click: {
                    $.root.childrens[0].text = format!("Clicked !");
                }
            }

            Repeater {
                id: repeater_example

                number item_count: 3

                on_ready: {
                    let count = $.this.item_count as u32;
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
                        println!("Runtime generated button {} clicked!", $.this.id);
                        println!("Test {}", $.repeater_example.childrens[0].id);
                        println!("Parent items count: {}", $.parent.item_count);
                        println!("This button index: {}", $.this.index);
                        println!("Parent child count: {}", $.parent.children_count);
                        println!("This child count: {}", $.this.children_count);
                        println!("Root child count: {}", $.root.children_count);

                        $.this.text = format!("Clicked ! {}", $.this.index);
                        $.root.childrens[0].text = format!("TESTOFTHEDEVIL Clicked ! {}", $.this.index);
                        $.repeater_example.childrens[0].text = format!("Clicked ! {}", $.this.index);
                    }
                }

                // todo
                // the basic idea works, but repeated items are in the repeater node
                // we need to place them in the parent of the repeater
                // or maybe, just allow the node in a repeater to be listed as children of the repeater parent
                // (could allow to use a repeater inside a Column or Row component)

                // todo
                // we must be able to add and remove items at runtime
                // repeater_example_delete_item(&mut engine, i);
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
