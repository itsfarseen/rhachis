use rhachis::{
    renderers::{Model, SimpleRenderer, TextureVertex, Transform, VertexSlice},
    Game, GameExt,
};

fn main() {
    Image::run();
}

struct Image {
    renderer: SimpleRenderer,
}

impl Game for Image {
    fn init(data: &rhachis::GameData) -> Self {
        let mut renderer = SimpleRenderer::new(data);
        renderer.models.push(Model::new(
            data,
            VertexSlice::TextureVertices(&[
                TextureVertex {
                    pos: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                },
                TextureVertex {
                    pos: [1.0, 0.0, 0.0],
                    tex_coords: [1.0, 0.0],
                },
                TextureVertex {
                    pos: [0.0, 1.0, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                TextureVertex {
                    pos: [1.0, 1.0, 0.0],
                    tex_coords: [1.0, 1.0],
                },
            ]),
            &[0, 1, 2, 1, 3, 2],
            &[Transform::default().matrix()],
        ));

        Self { renderer }
    }

    fn get_renderer(&mut self) -> &mut dyn rhachis::graphics::Renderer {
        &mut self.renderer
    }
}
