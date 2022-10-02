use rhachis::{
    renderers::{Model, SimpleProjection, SimpleRenderer, Texture, Transform},
    Game, GameExt,
};

#[rhachis::run]
struct Image {
    renderer: SimpleRenderer,
}

impl Game for Image {
    fn init(data: &rhachis::GameData) -> Self {
        let mut renderer = SimpleRenderer::new(data, SimpleProjection::Orthographic);
        renderer.models.push(Model::quad_texture(
            data,
            Texture::new(
                data,
                &image::open("examples/test.png").unwrap(),
                &renderer.linear_sampler,
            ),
            vec![Transform::scale((0.5, 0.5, 1.0))],
        ));

        Self { renderer }
    }

    fn get_renderer(&mut self) -> &mut dyn rhachis::graphics::Renderer {
        &mut self.renderer
    }
}
