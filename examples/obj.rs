use std::{f32::consts::TAU, time::Instant};

use glam::Mat4;
use rhachis::{
    renderers::{Model, SimpleRenderer, Transform},
    Game, GameExt,
};

#[rhachis::run]
struct Obj {
    renderer: SimpleRenderer,
}

impl Game for Obj {
    fn init(data: &rhachis::GameData) -> Self {
        data.window
            .lock()
            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        let window_size = data.window.lock().inner_size();

        let mut renderer = SimpleRenderer::new(
            data,
            Mat4::perspective_rh(
                TAU / 4.0,
                window_size.width as f32 / window_size.height as f32,
                0.1,
                100.0,
            ),
        );
        renderer.models.push(
            Model::from_obj(
                data,
                "examples/texture.obj",
                &renderer.nearest_sampler,
                vec![
                    Transform::translation((0.0, 0.0, -4.0).into()),
                    Transform::translation((0.0, 0.0, -4.0).into()),
                ],
            )
            .unwrap()
            .pop()
            .unwrap(),
        );

        Self { renderer }
    }

    fn update(&mut self, data: &rhachis::GameData) {
        if data.input.lock().is_key(
            rhachis::input::Key::Escape,
            rhachis::input::InputState::Pressed,
        ) {
            data.exit(None);
        }
        self.renderer.models[0].modify_transforms(|t| {
            t[0].set_x(f32::sin((Instant::now() - data.start_time).as_secs_f32() * 2.0) * 2.5);
            t[0].set_y(f32::sin((Instant::now() - data.start_time).as_secs_f32() * 2.0) * 2.5);
            t[1].set_x(
                -f32::sin(TAU / 3.0 + (Instant::now() - data.start_time).as_secs_f32() * 2.0) * 2.5,
            );
            t[1].set_y(
                f32::sin(TAU / 3.0 + (Instant::now() - data.start_time).as_secs_f32() * 2.0) * 2.5,
            );
        })
    }

    fn get_renderer(&mut self) -> &mut dyn rhachis::graphics::Renderer {
        &mut self.renderer
    }
}
