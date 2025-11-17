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
            number width: 500.0
            number height: 500.0

            Components::Button {
                id: counter_btn
                anchors: center
                number counter: 0
                text: { format!("Counter: {}", $.counter_btn.counter) }
                on_click: { $.counter_btn.counter += 1.0; }
            }
        }
    );

    println!("node from macro:\n {:#?}", engine.get_arena());

    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await
    }
}
