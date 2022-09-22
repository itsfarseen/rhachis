use glam::Vec2;
use image::{Rgba, RgbaImage};
use rhachis::{
    graphics::Renderer,
    math::lerp,
    rand::{perlin_2d, Noise},
    renderers::{Model, SimpleRenderer, Texture, Transform},
    *,
};

const IMAGE_WIDTH: u32 = 500;
const IMAGE_HEIGHT: u32 = 500;

#[rhachis::run]
struct PerlinImage(SimpleRenderer);

impl Game for PerlinImage {
    fn init(data: &rhachis::GameData) -> Self {
        let noise = Noise::new();
        let mut image = RgbaImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);

        for x in 0..IMAGE_WIDTH {
            for y in 0..IMAGE_HEIGHT {
                let perlin = perlin_2d(&noise, Vec2::new(x as f32, y as f32) / 30.0, lerp);
                let value = (perlin * 255.0) as u8;
                //let value = noise.get_range(x * IMAGE_HEIGHT + y, 0..256) as u8;
                image.put_pixel(x, y, Rgba([value, value, value, 255]));
            }
        }

        let mut renderer =
            SimpleRenderer::new(data, rhachis::renderers::SimpleProjection::Orthographic);
        let texture = Texture::new(data, &image.into(), &renderer.nearest_sampler);

        renderer.models.push(Model::quad_texture(
            data,
            texture,
            vec![Transform::default()],
        ));

        Self(renderer)
    }

    fn get_renderer(&mut self) -> &mut dyn Renderer {
        &mut self.0
    }
}
