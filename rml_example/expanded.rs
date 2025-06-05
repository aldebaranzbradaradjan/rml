Transformed input
Alias: Some(
    "Components",
)
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
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
fn main() {
    macroquad::Window::from_config(window_conf(), amain());
}
async fn amain() {
    let mut engine = (/*ERROR*/);
    {
        ::std::io::_print(
            format_args!("node from macro:\n {0:#?}\n", engine.get_arena()),
        );
    };
    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await
    }
}
Transformed input
Alias: Some(
    "Components",
)
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
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
fn main() {
    macroquad::Window::from_config(window_conf(), amain());
}
async fn amain() {
    let mut engine = (/*ERROR*/);
    {
        ::std::io::_print(
            format_args!("node from macro:\n {0:#?}\n", engine.get_arena()),
        );
    };
    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await
    }
}
Transformed input
Alias: Some(
    "Components",
)
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
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
fn main() {
    macroquad::Window::from_config(window_conf(), amain());
}
async fn amain() {
    let mut engine = (/*ERROR*/);
    {
        ::std::io::_print(
            format_args!("node from macro:\n {0:#?}\n", engine.get_arena()),
        );
    };
    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await
    }
}
Transformed input
Alias: Some(
    "Components",
)
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
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
fn main() {
    macroquad::Window::from_config(window_conf(), amain());
}
async fn amain() {
    let mut engine = (/*ERROR*/);
    {
        ::std::io::_print(
            format_args!("node from macro:\n {0:#?}\n", engine.get_arena()),
        );
    };
    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await
    }
}
Transformed input
Alias: Some(
    "Components",
)
Component name: Components::Card
Component name: Components::Button
Component name: Components::ButtonRed
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
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
fn main() {
    macroquad::Window::from_config(window_conf(), amain());
}
async fn amain() {
    let mut engine = (/*ERROR*/);
    {
        ::std::io::_print(
            format_args!("node from macro:\n {0:#?}\n", engine.get_arena()),
        );
    };
    loop {
        engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await
    }
}
