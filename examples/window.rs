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
