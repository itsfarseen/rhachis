# Rhachis

## `GameData` Struct

The `GameData` struct allows access to all core components of Rhachis. The components it carries are:

- Audio handler
- Delta time
- Graphics handler
- Input handler

These are all stored as a `Mutex<...>`.

## Graphics

The `Graphics` struct contains the core for rendering. This includes:

- Running pipelines
- Loading textures

There are many structures provided for graphics.

## Examples

### Blank Window

```rust
use rhachis::*;

fn main() {
    Window::run();
}

struct Window;

impl Game for Window {
    fn init(data: &mut GameData) -> Self {
        Self
    }
}
```



## Notes

- Making your own pipeline is easy and encouraged
  - Builtin pipelines are more for testing purposes
- Maths library agnostic
  - Probably only use `T: Into<[f32; 2]>` instead of `Vec2`, etc