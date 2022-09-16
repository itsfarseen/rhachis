use std::f32::consts::TAU;

use glam::{Mat4, Quat, Vec3};
use rhachis::{
    rand::Noise,
    renderers::{Model, SimpleRenderer, Transform},
    Game, GameExt, input::{Key, InputState}, GameData,
};

const TERRAIN_HEIGHT: u32 = 5;
const TERRAIN_WIDTH: u32 = 5;

fn make_projection(data: &GameData, distance: f32, angle: f32) -> Mat4 {
    let proj = Mat4::perspective_rh(
        TAU / 4.0,
        data.get_window_size().x as f32 / data.get_window_size().y as f32,
        0.1,
        100.0,
    );

    let view = Mat4::look_at_rh(
        Vec3::new(f32::sin(angle), f32::cos(angle), 1.0 / (distance / 5.0)) * distance,
        Vec3::ZERO,
        Vec3::Y
    );

    proj * view
}

#[rhachis::run]
struct PerlinExample {
    renderer: SimpleRenderer,
    cam_distance: f32,
    cam_angle: f32,
}

impl Game for PerlinExample {
    fn init(data: &GameData) -> Self {
        let cam_distance = 2.0;
        let cam_angle = 0.0;

        let mut renderer = SimpleRenderer::new(
            data,
            make_projection(data, cam_distance, cam_angle),
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

        Self {
            renderer,
            cam_distance,
            cam_angle,
        }
    }

    fn update(&mut self, data: &rhachis::GameData) {
        let input = data.input.lock();

        let mut cam_move = false;
        if input.is_key(Key::Char('w'), InputState::Down) && self.cam_distance > 1.0 {
            self.cam_distance -= 0.1;
            cam_move = true;
        }
        if input.is_key(Key::Char('s'), InputState::Down) {
            self.cam_distance += 0.1;
            cam_move = true;
        }

        if cam_move {
            let projection = make_projection(data, self.cam_distance, self.cam_angle);
            self.renderer.set_projection(data, projection);
        }
    }

    fn get_renderer(&mut self) -> &mut dyn rhachis::graphics::Renderer {
        &mut self.renderer
    }
}

fn terrain_transforms(noise: &Noise) -> Vec<Transform> {
    let mut to_ret = Vec::new();

    for x in 0..TERRAIN_WIDTH {
        for y in 0..TERRAIN_HEIGHT {
            let height = (x + y) as f32 / 3.0;

            to_ret.push(
                Transform::translation((
                    x as f32,
                    height - 2.0,
                    -(y as f32 + 3.0),
                ).into()).with_rotation(Quat::from_rotation_y(TAU / 4.0)).with_scale((0.5, 0.5, 0.5).into())
            );
        }
    }

    to_ret
}
