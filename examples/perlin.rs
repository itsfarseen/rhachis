use std::f32::consts::TAU;

use glam::{Mat4, Quat, Vec2, Vec3};
use rhachis::{
    input::{InputState, Key},
    math::smootherstep,
    rand::{perlin_2d, Noise},
    renderers::{Model, SimpleProjection, SimpleRenderer, Transform},
    Game, GameData, GameExt,
};

fn make_projection(data: &GameData, distance: f32, angle: f32) -> Mat4 {
    let proj = Mat4::perspective_rh(
        TAU / 4.0,
        data.get_window_size().x as f32 / data.get_window_size().y as f32,
        0.1,
        100.0,
    );

    let view = Mat4::look_at_rh(
        Vec3::new(f32::sin(angle), 1.0 / (distance / 5.0), f32::cos(angle)) * distance,
        Vec3::ZERO,
        Vec3::Y,
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
        data.window
            .lock()
            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        let cam_distance = 2.0;
        let cam_angle = 0.0;

        let mut renderer = SimpleRenderer::new(
            data,
            SimpleProjection::Other(make_projection(data, cam_distance, cam_angle)),
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
        let delta_time = data.delta_time.as_secs_f32();
        let input = data.input.lock();

        let mut cam_move = false;
        if input.is_key(Key::Char('w'), InputState::Down) && self.cam_distance > 1.0 {
            self.cam_distance -= 4.0 * delta_time;
            cam_move = true;
        }
        if input.is_key(Key::Char('s'), InputState::Down) {
            self.cam_distance += 4.0 * delta_time;
            cam_move = true;
        }
        if input.is_key(Key::Char('a'), InputState::Down) && self.cam_distance > 1.0 {
            self.cam_angle -= TAU / 2.0 * delta_time;
            cam_move = true;
        }
        if input.is_key(Key::Char('d'), InputState::Down) {
            self.cam_angle += TAU / 2.0 * delta_time;
            cam_move = true;
        }
        if input.is_key(Key::Char('r'), InputState::Pressed) {
            self.renderer.models[0].set_transforms(terrain_transforms(&Noise::new()));
        }
        if input.is_key(Key::Escape, InputState::Pressed) {
            data.exit(None);
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

    for x in 0..90 {
        for y in 0..90 {
            let pos = Vec2::new(x as f32, y as f32);
            let height = (perlin_2d(noise, pos / 20.0, smootherstep) * 10.0).floor()
                + (perlin_2d(noise, pos / 10.0, smootherstep) * 3.0).floor();

            to_ret.push(
                Transform::translation((x as f32, height - 2.0, -(y as f32 + 3.0)).into())
                    .with_rotation(Quat::from_rotation_y(TAU / 4.0))
                    .with_scale((0.5, 0.5, 0.5).into()),
            );
        }
    }

    to_ret
}
