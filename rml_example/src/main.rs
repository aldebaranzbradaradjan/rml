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

            Components::Button {
                id: counter_btn
                anchors: center
                number counter: 0
                text: { format!("Counter: {}", $.counter_btn.counter) }
                on_click: { $.counter_btn.counter += 1.0; }
                font: "liberation"
            }
        }
    );

    
    let font = load_ttf_font("./LiberationSerif-Regular.ttf")
        .await
        .unwrap();
    engine.add_font("liberation".to_string(), font);

    let texture = load_texture("./Adriaen_van_Ostade_006.png").await.unwrap();
    engine.add_texture("Adriaen".to_string(), texture);


    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_root(&mut engine);
        next_frame().await
    }
}
