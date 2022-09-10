use rhachis::{graphics::Renderer, *};

fn main() {
}

struct Tri {
    renderer: TriRenderer,
}

impl Game for Tri {
    fn init(data: &GameData) -> Self {
        Tri { renderer: TriRenderer }
    }

    fn get_renderer(&mut self) -> &mut dyn Renderer {
        &mut self.renderer
    }
}

struct TriRenderer;

impl Renderer for TriRenderer {}
