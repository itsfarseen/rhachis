use glam::{Vec2, Vec3};
use image::{Rgba, RgbaImage};
use rhachis::{
    graphics::Renderer,
    math::lerp,
    rand::{perlin_2d, Noise},
    renderers::{Model, SimpleRenderer, Texture, Transform},
    *,
};

const IMAGE_WIDTH: u32 = 1200;
const IMAGE_HEIGHT: u32 = 800;

#[rhachis::run]
struct PerlinImage(SimpleRenderer);

impl Game for PerlinImage {
    fn init(data: &rhachis::GameData) -> Self {
        data.set_window_size((IMAGE_WIDTH, IMAGE_HEIGHT).into());

        let noise = Noise::new();
        let mut image = RgbaImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);

        for x in 0..IMAGE_WIDTH {
            for y in 0..IMAGE_HEIGHT {
                let perlin = perlin_2d(&noise, Vec2::new(x as f32, y as f32) / 64.0, lerp);
                let value = ((perlin + 1.0) * 127.0) as u8;
                image.put_pixel(x, y, Rgba([value, value, value, 255]));
            }
        }

        let mut renderer =
            SimpleRenderer::new(data, rhachis::renderers::SimpleProjection::Orthographic);
        let texture = Texture::new(data, &image.into(), &renderer.nearest_sampler);

        renderer.models.push(Model::quad_texture(
            data,
            texture,
            vec![
                Transform::scale(Vec3::new(2.0, 2.0, 1.0)).with_translation(Vec3::new(-1.0, -1.0, 0.0))
            ],
        ));

        Self(renderer)
    }

    fn get_renderer(&mut self) -> &mut dyn Renderer {
        &mut self.0
    }
}
