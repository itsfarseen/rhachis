use rhachis::{graphics::Renderer, renderers::SimpleRenderer, *};

fn main() {
    Tri::run();
}

struct Tri {
    renderer: SimpleRenderer,
}

impl Game for Tri {
    fn init(data: &GameData) -> Self {
        Tri {
            renderer: SimpleRenderer::new(data),
        }
    }

    fn get_renderer(&mut self) -> &mut dyn Renderer {
        &mut self.renderer
    }
}
