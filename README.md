# rml
A basic QML like toolkit in Rust

## Why
This project is a small playground for testing ideas and learning. It's not intended for production use.

## Example
```rust
let mut engine = rml!(
    Node {
        id: root
        width: 500.0
        height: 500.0

        // draw a rectangle at the center of the screen
        // In this rectangle, draw 4 Rectangles to create border of 10 px,
        // then draw rectangle in the right bottom edge to draw a simplifier RML letters
        Rectangle {
            id: outer_rect
            x: {
                let root_width = get_number!(engine, root, width);
                let outer_rect_width = get_number!(engine, outer_rect, width);
                let outer_rect_x = root_width / 2.0 - outer_rect_width / 2.0;
                outer_rect_x
            }
            y: {
                let root_height = get_number!(engine, root, height);
                let outer_rect_height = get_number!(engine, outer_rect, height);
                let outer_rect_y = root_height / 2.0 - outer_rect_height / 2.0;
                outer_rect_y
            }
            width: 200
            height: 200
            color: "rgba(1.0, 0.0, 0.0, 1.0)"

            fn set_text() {
                let x = engine.get_number_property_of_node("outer_rect", "x", 0.0);
                let y = engine.get_number_property_of_node("outer_rect", "y", 0.0);
                let text = format!("rml {x} {y}");
                set_string!(engine, text, text, text);
            }

            on_x_changed: { set_text() }
            on_y_changed: { set_text() }
            
            Rectangle {
                id: inner_rect
                x: {
                    let outer_rect_width = get_number!(engine, outer_rect, width);
                    let inner_rect_width = get_number!(engine, inner_rect, width);
                    let inner_rect_x = outer_rect_width / 2.0 - inner_rect_width / 2.0;
                    inner_rect_x
                }
                y: {
                    let outer_rect_height = get_number!(engine, outer_rect, height);
                    let inner_rect_height = get_number!(engine, inner_rect, height);
                    let inner_rect_y = outer_rect_height / 2.0 - inner_rect_height / 2.0;
                    inner_rect_y
                }
                width: 160
                height: 160
                color: "rgba(0.3, 0.5, 0.3, 1.0)"

                Rectangle {
                    id: inner_inner_rect
                    x: 150
                    y: 150
                    width: 10
                    height: 10
                    color: "rgba(0.0, 0., 0.3, 1.0)"
                }

                Node {
                    id: inner_inner_node
                    x: 0
                    y: 0
                    width: 100
                    height: 100

                    Text {
                        id: text
                        x: 0
                        y: 0
                        width: 100
                        height: 100
                        color: "rgba(0.0, 0.0, 1.0, 1.0)"
                        text: "rml"
                        font_size: 20
                    }
                }
            }
        }
    }
);
```

## Current state: Unstable
Code may be incomplete, buggy, or broken. Use at your own risk!

## License
It's MIT.

That's it!