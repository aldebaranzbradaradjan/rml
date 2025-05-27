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

        ...

        // check example to the full code

        ...


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

### Existing
* A simple DSL supporting Rectangle and Text nodes
* Manages properties of the following types: Number, Bool, String, Vec
* A core engine that handles the node arena, event system, properties, and callbacks
* Allows defining callbacks, functions, and initial values in Rust directly within the DSL
* Basic anchoring system
* Rendering powered by macroquad
* Designed to be simple, easy to extend, and modify

### Todo
* Renamed from RML to CML (Cute Markup Language) for a lighter, more playful tone
* Support for loading multiple files (e.g., component definitions in separate files)
* System event handling:
    * Keyboard and mouse input
    * Window-level events
    * Ability to define and dispatch custom events
* Resource system:
    * Simple way to declare and include assets (e.g., images, fonts) from a specified folder
    * Assets can be bundled and shipped with the executable
* (Optional) ID scoping: Limit ID visibility to their respective files to prevent conflicts
* Positioning widgets:
    * Support for layout primitives: Column, Row, Grid
    * Fully compatible with anchors
* Layout system:
    * Coordinate positioning with layout primitives (Column, Row, Grid)
    * Integrates cleanly with the anchoring system
* Data models:
    * Declarative macros for ListModel, TableModel based on Rust structs
    * Support for passing fields, sorting, and filtering

## License
It's MIT.

That's it!