// Test mouse wheel delta values and scroll direction

use macroquad::prelude::*;

use std::collections::HashMap;
use rml_core::prelude::*; 
use rml_macros::rml;

fn window_conf() -> Conf {
    Conf {
        window_title: "RML MouseArea Test".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut engine = rml!(
        Node {
            id: root
            anchors: fill
            color color: { DARKGRAY }

            // Combined scroll area - Both directions
            Rectangle {
                id: scroll_area
                anchors: top | left
                margins: 20.0
                number width: 300.0
                number height: 150.0
                color color: { Color::new(0.6, 0.8, 0.3, 1.0) }

                Text {
                    x: 20.0
                    y: 30.0
                    color color: { Color::new(0.0, 0.0, 0.0, 1.0) }
                    text: "Combined Scroll Area"
                    font_size: 20
                }
                
                Text {
                    id: c_scroll_info
                    x: 20.0
                    y: 60.0
                    color color: { Color::new(0.0, 0.0, 0.0, 1.0) }
                    string text: "Both directions affect size"
                    font_size: 15
                }

                MouseArea {
                    anchors: fill
                    
                    on_mouse_wheel: {
                        let delta_x = get_mouse_wheel_delta_x!(engine);
                        let delta_y = get_mouse_wheel_delta_y!(engine);
                        
                        println!("Combined area - Wheel delta: x={}, y={}", delta_x, delta_y);
                        
                        // Use combined delta for scaling
                        let combined_delta = (delta_x + delta_y) * 10.0;
                        
                        // Update both width and height
                        let base_width = $.scroll_area.width;
                        let base_height = $.scroll_area.height;
                        $.scroll_area.width = base_width + combined_delta;
                        $.scroll_area.height = base_height + combined_delta;
                        
                        $.c_scroll_info.text = format!("delta x: {:.1}, delta y: {:.1}", delta_x, delta_y);
                    }
                    
                    on_mouse_enter: {
                        $.scroll_area.color = Color::new(0.7, 0.9, 0.4, 1.0);
                    }
                    
                    on_mouse_leave: {
                        $.scroll_area.color = Color::new(0.6, 0.8, 0.3, 1.0);
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
                color color: { Color::new(0.8, 0.3, 0.3, 1.0) }

                Text {
                    anchors: center
                    color color: { Color::new(1.0, 1.0, 1.0, 1.0) }
                    text: "Reset Size"
                    font_size: 18
                }

                MouseArea {
                    anchors: fill
                    
                    on_mouse_down: {
                        println!("Reset button clicked");
                        $.scroll_area.width = 300.0;
                        $.scroll_area.height = 150.0;
                        $.reset_button.color = Color::new(0.7, 0.2, 0.2, 1.0);
                        $.c_scroll_info.text = "Size reset to default".to_string();
                    }

                    on_mouse_up: {
                        println!("Reset button released");
                        $.reset_button.color = Color::new(0.8, 0.3, 0.3, 1.0);
                    }
                    
                    on_mouse_enter: {
                        $.reset_button.color = Color::new(0.9, 0.4, 0.4, 1.0);
                    }
                    
                    on_mouse_leave: {
                        $.reset_button.color = Color::new(0.8, 0.3, 0.3, 1.0);
                    }
                }
            }

            // Draggable element to test computed positions
            Rectangle {
                id: draggable
                number x: 200.0
                number y: 100.0
                width: 80.0
                height: 80.0
                color color: { Color::new(0.3, 0.8, 0.6, 1.0) }
                bool is_dragging: false

                Text {
                    id: drag_text
                    anchors: center
                    color color: { Color::new(0.0, 0.0, 0.0, 1.0) }
                    text: "Drag"
                    font_size: 15
                }

                MouseArea {
                    id: drag_area
                    anchors: fill
                    consume_mouse_enter: true
                    
                    on_mouse_down: {
                        println!("Drag started");
                        $.draggable.is_dragging = true;
                    }
                    
                    on_mouse_up: {
                        println!("Drag ended");
                        $.draggable.is_dragging = false;
                    }

                    on_mouse_move: {
                        if $.draggable.is_dragging {
                            let (mouse_x, mouse_y) = engine.get_mouse_position();

                            // Convert to container-relative coordinates
                            let container_abs_x = $.root.computed_x;
                            let container_abs_y = $.root.computed_y;
                            let new_x = mouse_x - container_abs_x - 40.0; // Center on cursor
                            let new_y = mouse_y - container_abs_y - 40.0;
                            $.draggable.x = new_x;
                            $.draggable.y = new_y;
                        }
                    }

                    on_mouse_enter: {
                        $.draggable.color = Color::new(0.4, 0.9, 0.7, 1.0);
                    }
                    
                    on_mouse_leave: {
                        $.draggable.color = Color::new(0.3, 0.8, 0.6, 1.0);
                    }
                }
            }

            // Container to test key events
            Rectangle {
                id: test_key
                anchors: top | right
                margins: 20.0
                number width: 300.0
                number height: 150.0
                color color: { Color::new(0.6, 0.8, 0.3, 1.0) }

                Text {
                    id: test_key_text
                    x: 20.0
                    y: 30.0
                    color color: { Color::new(1.0, 1.0, 1.0, 1.0) }
                    string text: "Container for Key Events"
                    font_size: 20
                }

                on_key_pressed: {
                    let key = get_key_event!(engine);
                    $.test_key_text.text = format!("key pressed : {:?}", key);
                    println!("Key pressed in container: {:?}", key);
                }

                MouseArea {
                    anchors: fill
                    
                    on_mouse_enter: {
                        $.test_key.color = Color::new(0.3, 0.4, 0.9, 1.0);
                    }
                    
                    on_mouse_leave: {
                        $.test_key.color = Color::new(0.2, 0.3, 0.8, 1.0);
                    }

                    on_mouse_down : {
                        $.test_key.color = Color::new(0.1, 0.2, 0.7, 1.0);
                    }

                    on_mouse_up : {
                        $.test_key.color = Color::new(0.2, 0.3, 0.8, 1.0);
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
        engine.process_events();
        rml_core::draw::draw_root(&mut engine);
        next_frame().await;
    }
}