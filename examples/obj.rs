use std::{f32::consts::TAU, time::Instant};

use rhachis::{
    renderers::{Model, SimpleProjection, SimpleRenderer, Transform},
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

        let projection = SimpleProjection::new_perspective(data);
        let mut renderer = SimpleRenderer::new(data, projection);
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
            *t[0].x_mut() = f32::sin((Instant::now() - data.start_time).as_secs_f32() * 2.0) * 2.5;
            *t[0].y_mut() = f32::sin((Instant::now() - data.start_time).as_secs_f32() * 2.0) * 2.5;
            *t[1].x_mut() =
                -f32::sin(TAU / 3.0 + (Instant::now() - data.start_time).as_secs_f32() * 2.0) * 2.5;
            *t[1].y_mut() =
                f32::sin(TAU / 3.0 + (Instant::now() - data.start_time).as_secs_f32() * 2.0) * 2.5;
        })
    }

    fn get_renderer(&mut self) -> &mut dyn rhachis::graphics::Renderer {
        &mut self.renderer
    }
}
