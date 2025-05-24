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

        // a rectangle anchored to fill the root node
        Rectangle {
            width: 10
            height: 10
            anchors: fill
            color: "rgba(1.0, 0.0, 0.0, 0.3)"
        }
        
        // a rectangle anchored to fill the root node, but with margins
        Rectangle {
            width: 10
            height: 10
            anchors: fill
            margins: 50
            color: "rgba(0.0, 1.0, 0.0, 0.3)"
        }

        // a rectangle anchored in top left corner of the root node
        Rectangle {
            width: 10
            height: 10
            anchors: top | left
            color: "rgba(0.0, 1.0, 0.0, 1.0)"
        }

        // a rectangle anchored in top right corner of the root node
        Rectangle {
            width: 10
            height: 10
            anchors: top | right
            color: "rgba(0.0, 0.0, 1.0, 1.0)"
        }

        // a rectangle anchored in bottom left corner of the root node
        Rectangle {
            width: 10
            height: 10
            anchors: bottom | left
            color: "rgba(1.0, 1.0, 0.0, 1.0)"
        }

        // a rectangle anchored in bottom right corner of the root node
        Rectangle {
            width: 10
            height: 10
            anchors: bottom | right
            color: "rgba(1.0, 0.0, 1.0, 1.0)"
        }

        // a rectangle anchored at left and right of the root node and centered vertically
        Rectangle {
            width: 10
            height: 10
            anchors: left | right | vertical_center
            color: "rgba(0.5, 0.0, 0.5, 1.0)"
        }

        // a rectangle anchored at top and bottom of the root node and centered horizontally
        Rectangle {
            width: 10
            height: 10
            anchors: top | bottom | horizontal_center
            color: "rgba(0.5, 0.5, 0.0, 1.0)"
        }

        // a rectangle anchored in the center of the root node
        Rectangle {
            width: 10
            height: 10
            anchors: center
            color: "rgba(0.5, 0.7, 0.5, 1.0)"
        }

        // a text node anchored to the top left corner of the root node with a margin and a width specified
        Text {
            anchors: top | left
            margins: 20
            width: 30
            color: "rgba(1.0, 1.0, 1.0, 1.0)"
            text: "Test Text"
            font_size: 20
        }

        Text {
            x: 20
            y: 0
            color: "rgba(1.0, 1.0, 1.0, 1.0)"
            text: "Test Text"
            font_size: 20
        }

        // a smiling face made of rectangles
        Rectangle {
            anchors: right | top
            margins: 20
            width: 100
            height: 100
            color: "rgba(1.0, 1.0, 1.0, 1.0)"
            
            Rectangle {
                anchors: left | top
                margins: 30
                width: 10
                height: 10
                color: "rgba(0., 0., 0., 1.0)"
            }
            Rectangle {
                anchors: right | top
                margins: 30
                width: 10
                height: 10
                color: "rgba(0., 0., 0., 1.0)"
            }
            Rectangle {
                anchors: bottom | horizontal_center
                bottom_margin: 30
                width: 70
                height: 10
                color: "rgba(0., 0., 0., 1.0)"

                Rectangle {
                    anchors: left | top
                    left_margin: -10
                    top_margin: -10
                    width: 10
                    height: 10
                    color: "rgba(0., 0., 0., 1.0)"
                }
                Rectangle {
                    anchors: right | top
                    right_margin: -10
                    top_margin: -10
                    width: 10
                    height: 10
                    color: "rgba(0., 0., 0., 1.0)"
                }
            }
        }

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
            color: "rgba(1.0, 1.0, 1.0, 1.0)"

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
                    outer_rect_width / 2.0 - inner_rect_width / 2.0
                }
                y: {
                    let outer_rect_height = get_number!(engine, outer_rect, height);
                    let inner_rect_height = get_number!(engine, inner_rect, height);
                    outer_rect_height / 2.0 - inner_rect_height / 2.0
                }
                
                width: 160
                height: 160
                color: "rgba(0.3, 0.3, 0.3, 1.0)"

                Rectangle {
                    id: left_eye
                    x: 30
                    y: 30
                    width: 10
                    height: 10
                    color: "rgba(0., 0., 0., 1.0)"
                }

                Rectangle {
                    id: right_eye
                    x: 120
                    y: 30
                    width: 10
                    height: 10
                    color: "rgba(0., 0., 0., 1.0)"
                }

                Rectangle {
                    id: mouth

                    anchors: center
                    x: 30
                    y: 100
                    width: 100
                    height: 10
                    color: "rgba(0., 0., 0., 1.0)"

                    Rectangle {
                        id: mouth_l
                        x: -10
                        y: -10
                        width: 10
                        height: 10
                        color: "rgba(0., 0., 0., 1.0)"
                    }

                    Rectangle {
                        id: mouth_r
                        x: 100
                        y: -10
                        width: 10
                        height: 10
                        color: "rgba(0., 0., 0., 1.0)"
                    }
                }

                Node {
                    id: inner_inner_node
                    x: 10
                    y: 0
                    width: 100
                    height: 100

                    Text {
                        id: text
                        x: 0
                        y: 5
                        width: 100
                        height: 100
                        color: "rgba(1.0, 1.0, 1.0, 1.0)"
                        text: "rml"
                        font_size: 20
                    }
                }

                Rectangle {
                    anchors: left | right | bottom
                    bottom_margin: 10
                    width: 10
                    height: 10
                    color: "rgba(1., 0., 0., 1.0)"
                }
            }
        }
    }
);
```
### Result
![example result](rml_example/example_smile.png)

## Current state: Unstable
Code may be incomplete, buggy, or broken. Use at your own risk!

## License
It's MIT.

That's it!