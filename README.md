[![Crates.io](https://img.shields.io/crates/v/rhachis?style=flat-square)](https://crates.io/crates/rhachis)
[![Crates.io](https://img.shields.io/crates/d/rhachis?style=flat-square)](https://crates.io/crates/rhachis)
[![Docs.rs](https://img.shields.io/docsrs/rhachis?style=flat-square)](https://docs.rs/rhachis)

# Rhachis

Rhachis is a Rust framework primarily intended for making games. It intends to be as simple as possible, while still allowing as much customisation and power writing the engine yourself can.

## Example

This example shows the bare minimum required to make a program start at all in Rhachis.

```rust
use rhachis::{graphics::EmptyRenderer, *};

#[rhachis::run]
struct Window(EmptyRenderer);

impl Game for Window {
    fn init(_: &GameData) -> Self {
        Self(EmptyRenderer)
    }

    fn get_renderer(&mut self) -> &mut dyn graphics::Renderer {
        &mut self.0
    }
}
```

More in depth examples can be found in the repository's [examples directory](https://github.com/SalsaGal/rhachis/tree/master/examples).
