use glam::Mat4;
use rhachis::{
    renderers::{Model, SimpleRenderer, Texture, Transform},
    Game, GameExt,
};

#[rhachis::run]
struct Image {
    renderer: SimpleRenderer,
}

impl Game for Image {
    fn init(data: &rhachis::GameData) -> Self {
        let mut renderer = SimpleRenderer::new(
            data,
            Mat4::orthographic_lh(-2.0, 2.0, -1.0, 1.0, -0.1, 100.0),
        );
        renderer.models.push(Model::quad_texture(
            data,
            Texture::new(
                data,
                &image::open("examples/test.png").unwrap(),
                &renderer.linear_sampler,
            ),
            vec![Transform::scale((0.5, 0.5, 1.0).into())],
        ));

        Self { renderer }
    }

    fn get_renderer(&mut self) -> &mut dyn rhachis::graphics::Renderer {
        &mut self.renderer
    }
}
