// Test mouse wheel delta values and scroll direction

use macroquad::prelude::*;

use std::collections::HashMap;
use rml_core::{ RmlEngine, Property, AbstractValue, get_value, get_number, set_number, get_string, set_string, get_bool, set_bool, get_computed_x, get_computed_y, get_computed_width, get_computed_height, ItemTypeEnum, EventType, SystemEvent, get_mouse_wheel_delta_x, get_mouse_wheel_delta_y, get_mouse_event_pos, get_key_event};
use rml_macros::rml;

fn window_conf() -> Conf {
    Conf {
        window_title: "RML MouseArea Test".to_owned(),
        window_width: 800,
        window_height: 600,
        window_resizable: false,
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
        Node {
            id: root
            width: 800.0
            height: 600.0

            // Background
            Rectangle {
                anchors: fill
                color: "rgba(0.1, 0.1, 0.2, 1.0)"
            }

            // Combined scroll area - Both directions
            Rectangle {
                id: scroll_area
                anchors: top | left
                margins: 20.0
                width: 300.0
                height: 150.0
                color: "rgba(0.6, 0.8, 0.3, 1.0)"

                Text {
                    x: 20.0
                    y: 30.0
                    color: "rgba(0.0, 0.0, 0.0, 1.0)"
                    text: "Combined Scroll Area"
                    font_size: 20
                }
                
                Text {
                    id: c_scroll_info
                    x: 20.0
                    y: 60.0
                    color: "rgba(0.0, 0.0, 0.0, 1.0)"
                    text: "Both directions affect size"
                    font_size: 15
                }

                MouseArea {
                    anchors: fill
                    
                    on_mouse_wheel: {
                        let delta_x = get_mouse_wheel_delta_x!(engine);
                        let delta_y = get_mouse_wheel_delta_y!(engine);
                        
                        println!("Combined area - Wheel delta: x={}, y={}", delta_x, delta_y);
                        
                        // Use combined delta for scaling
                        let combined_delta = delta_x + delta_y;
                        
                        // Update both width and height
                        let base_width = get_number!(engine, scroll_area, width);
                        let base_height = get_number!(engine, scroll_area, height);
                        set_number!(engine, scroll_area, width, base_width + combined_delta);
                        set_number!(engine, scroll_area, height, base_height + combined_delta);
                        
                        set_string!(engine, c_scroll_info, text, format!("delta x: {:.1}, delta y: {:.1}", delta_x, delta_y));
                    }
                    
                    on_mouse_enter: {
                        set_string!(engine, scroll_area, color, "rgba(0.7, 0.9, 0.4, 1.0)".to_string());
                    }
                    
                    on_mouse_leave: {
                        set_string!(engine, scroll_area, color, "rgba(0.6, 0.8, 0.3, 1.0)".to_string());
                    }
                }
            }

            // Button to reset scroll area size
            Rectangle {
                id: reset_button
                anchors: bottom | left
                margins: 20.0
                width: 120.0
                height: 40.0
                color: "rgba(0.8, 0.3, 0.3, 1.0)"

                Text {
                    anchors: center
                    color: "rgba(1.0, 1.0, 1.0, 1.0)"
                    text: "Reset Size"
                    font_size: 18
                }

                MouseArea {
                    anchors: fill
                    
                    on_mouse_down: {
                        println!("Reset button clicked");
                        set_number!(engine, scroll_area, width, 300.0);
                        set_number!(engine, scroll_area, height, 150.0);
                        set_string!(engine, reset_button, color, "rgba(0.7, 0.2, 0.2, 1.0)".to_string());
                        set_string!(engine, c_scroll_info, text, "Size reset to default".to_string());
                    }

                    on_mouse_up: {
                        println!("Reset button released");
                        set_string!(engine, reset_button, color, "rgba(0.8, 0.3, 0.3, 1.0)".to_string());
                    }
                    
                    on_mouse_enter: {
                        set_string!(engine, reset_button, color, "rgba(0.9, 0.4, 0.4, 1.0)".to_string());
                    }
                    
                    on_mouse_leave: {
                        set_string!(engine, reset_button, color, "rgba(0.8, 0.3, 0.3, 1.0)".to_string());
                    }
                }
            }

            // Draggable element to test computed positions
            Rectangle {
                id: draggable
                x: 200.0
                y: 100.0
                width: 80.0
                height: 80.0
                color: "rgba(0.3, 0.8, 0.6, 1.0)"
                is_dragging: false

                Text {
                    id: drag_text
                    anchors: center
                    color: "rgba(0.0, 0.0, 0.0, 1.0)"
                    text: "Drag"
                    font_size: 15
                }

                MouseArea {
                    id: drag_area
                    anchors: fill
                    consume_mouse_enter: true
                    
                    on_mouse_down: {
                        println!("Drag started");
                        set_bool!(engine, draggable, is_dragging, true);
                    }
                    
                    on_mouse_up: {
                        println!("Drag ended");
                        set_bool!(engine, draggable, is_dragging, false);
                    }

                    on_mouse_move: {
                        if get_bool!(engine, draggable, is_dragging) {
                            let (mouse_x, mouse_y) = engine.get_mouse_position();
                            // Convert to container-relative coordinates
                            let container_abs_x = get_computed_x!(engine, container);
                            let container_abs_y = get_computed_y!(engine, container);
                            
                            let new_x = mouse_x - container_abs_x - 40.0; // Center on cursor
                            let new_y = mouse_y - container_abs_y - 40.0;
                            
                            set_number!(engine, draggable, x, new_x);
                            set_number!(engine, draggable, y, new_y);

                            // todo add this syntax to rml_macros
                            // $.draggable.x = new_x;
                            // $.draggable.y = new_y;
                        }
                    }

                    on_mouse_enter: {
                        set_string!(engine, draggable, color, "rgba(0.4, 0.9, 0.7, 1.0)".to_string());
                    }
                    
                    on_mouse_leave: {
                        set_string!(engine, draggable, color, "rgba(0.3, 0.8, 0.6, 1.0)".to_string());
                    }
                }
            }

            // Container to test key events
            Rectangle {
                id: test_key
                anchors: top | right
                margins: 20.0
                width: 300.0
                height: 200.0
                color: "rgba(0.2, 0.3, 0.8, 1.0)"

                Text {
                    id: test_key_text
                    x: 20.0
                    y: 30.0
                    color: "rgba(1.0, 1.0, 1.0, 1.0)"
                    text: "Container for Key Events"
                    font_size: 20
                }

                on_key_pressed: {
                    let key = get_key_event!(engine);
                    set_string!(engine, test_key_text, text, format!("key pressed : {:?}", key));
                    println!("Key pressed in container: {:?}", key);
                }

                MouseArea {
                    anchors: fill
                    
                    on_mouse_enter: {
                        set_string!(engine, test_key, color, "rgba(0.3, 0.4, 0.9, 1.0)".to_string());
                    }
                    
                    on_mouse_leave: {
                        set_string!(engine, test_key, color, "rgba(0.2, 0.3, 0.8, 1.0)".to_string());
                    }

                    on_mouse_down : {
                        set_string!(engine, test_key, color, "rgba(0.1, 0.2, 0.7, 1.0)".to_string());
                    }

                    on_mouse_up : {
                        set_string!(engine, test_key, color, "rgba(0.2, 0.3, 0.8, 1.0)".to_string());
                    }

                    on_click: {
                        println!("Container clicked");
                        engine.set_focused_node("test_key");
                    }
                }
            }
        }
    );

    println!("=== RML Scroll Test ===");
    println!("Test mouse wheel delta values");
    println!("Console shows exact delta values for debugging");

    loop {
        let _events = engine.process_events();
        clear_background(BLACK);
        rml_core::draw::draw_childs(&mut engine, "root", (0., 0.));
        next_frame().await;
    }
}