use rhachis::{
    graphics::Renderer,
    renderers::{ColorVertex, Model, SimpleRenderer, Transform},
    *,
};

fn main() {
    Tri::run();
}

struct Tri {
    renderer: SimpleRenderer,
}

impl Game for Tri {
    fn init(data: &GameData) -> Self {
        let mut renderer = SimpleRenderer::new(data);
        renderer.models.push(Model::new_color(
            data,
            &[
                ColorVertex {
                    pos: [0.0, 0.0, 0.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                ColorVertex {
                    pos: [1.0, 0.0, 0.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                ColorVertex {
                    pos: [1.0, 1.0, 0.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
            ],
            &[0, 1, 2],
            Transform::default(),
        ));

        Tri { renderer }
    }

    fn get_renderer(&mut self) -> &mut dyn Renderer {
        &mut self.renderer
    }
}
