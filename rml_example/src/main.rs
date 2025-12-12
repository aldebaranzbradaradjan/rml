// This example demonstrates how to use the RML library to create a simple 2D GUI
use rml_core::prelude::*;
use rml_macros::rml;

fn window_conf() -> Conf {
    Conf {
        window_title: "RML Example".to_owned(),
        window_width: 500,
        window_height: 500,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut engine = rml!(
        import "components" as Components

        Node {
            id: root
            anchors: fill
            color color: { DARKGRAY }

            Texture {
                id: background_texture
                anchors: fill
                margins: 10
                source: "Adriaen"
                keep_aspect_ratio: true
            }

            Components::Column {
                id: column
                anchors: center

                Components::Button {
                    number counter: 0
                    number top_margin: 0
                    text: { format!("Counter: {}", $.this.counter) }
                    on_click: { $.this.counter += 1.0; println!("Clicked ! {}", $.this.counter); }
                    font: "liberation"
                }

                Components::Button {
                    number counter: 0
                    number top_margin: 0
                    text: { format!("Counter: {}", $.this.counter) }
                    on_click: { $.this.counter += 1.0; }
                    font: "liberation"
                }
            }
        }
    );
    
    let font = load_ttf_font("./LiberationSerif-Regular.ttf").await.unwrap();
    engine.add_font("liberation".to_string(), font);

    let texture = load_texture("./Adriaen_van_Ostade_006.png").await.unwrap();
    engine.add_texture("Adriaen".to_string(), texture);

    loop {
        engine.process_events();
        rml_core::draw::draw_root(&mut engine);
        next_frame().await
    }
}
