use rhachis::{
    graphics::Renderer,
    renderers::{ColorVertex, Model, SimpleRenderer, Transform, VertexSlice},
    *,
};

#[rhachis::run]
struct Tri {
    renderer: SimpleRenderer,
}

impl Game for Tri {
    fn init(data: &GameData) -> Self {
        let mut renderer = SimpleRenderer::new(data, renderers::SimpleProjection::Orthographic);
        renderer.models.push(Model::new(
            data,
            VertexSlice::ColorVertices(&[
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
            ]),
            &[0, 1, 2],
            vec![Transform::default()],
        ));

        Tri { renderer }
    }

    fn get_renderer(&mut self) -> &mut dyn Renderer {
        &mut self.renderer
    }
}
