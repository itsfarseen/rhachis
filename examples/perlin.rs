use glam::Mat4;
use rhachis::{
    rand::Noise,
    renderers::{Model, SimpleRenderer, Transform},
    Game, GameExt,
};

const TERRAIN_HEIGHT: u32 = 5;
const TERRAIN_WIDTH: u32 = 5;

#[rhachis::run]
struct PerlinExample {
    renderer: SimpleRenderer,
}

impl Game for PerlinExample {
    fn init(data: &rhachis::GameData) -> Self {
        let mut renderer = SimpleRenderer::new(
            data,
            Mat4::orthographic_lh(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0),
        );

        renderer.models.push(
            Model::from_obj(
                data,
                "examples/cube.obj",
                &renderer.nearest_sampler,
                terrain_transforms(&Noise::new()),
            )
            .unwrap()
            .pop()
            .unwrap(),
        );

        Self { renderer }
    }

    fn get_renderer(&mut self) -> &mut dyn rhachis::graphics::Renderer {
        &mut self.renderer
    }
}

fn terrain_transforms(noise: &Noise) -> Vec<Transform> {
    let mut to_ret = Vec::new();

    for x in 0..TERRAIN_WIDTH {
        for y in 0..TERRAIN_HEIGHT {
            let height = (x + y) as f32;

            to_ret.push(Transform::translation((
                x as f32,
                height,
                y as f32,
            ).into()));
        }
    }

    to_ret
}
