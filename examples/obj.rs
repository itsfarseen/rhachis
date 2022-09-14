use std::f32::consts::TAU;

use glam::{Mat4, Quat};
use rhachis::{
    renderers::{Model, SimpleRenderer, Transform},
    Game, GameExt,
};

#[rhachis::run]
struct Obj {
    renderer: SimpleRenderer,
    angle: f32,
}

impl Game for Obj {
    fn init(data: &rhachis::GameData) -> Self {
        let window_size = data.window.lock().inner_size();

        let mut renderer = SimpleRenderer::new(
            data,
            Mat4::perspective_lh(
                TAU / 4.0,
                window_size.width as f32 / window_size.height as f32,
                0.1,
                100.0,
            ),
        );
        renderer.models.push(
            Model::from_obj(data, "examples/texture.obj", &renderer.nearest_sampler)
                .unwrap()
                .pop()
                .unwrap()
                .with_transforms(vec![Transform::translation((0.0, 0.0, 4.0).into())]),
        );

        Self {
            renderer,
            angle: 0.0,
        }
    }

    fn update(&mut self, _: &rhachis::GameData) {
        self.angle += 0.1;
        self.renderer.models[0].modify_transforms(|t| t[0].set_rotation(Quat::from_rotation_x(self.angle)))
    }

    fn get_renderer(&mut self) -> &mut dyn rhachis::graphics::Renderer {
        &mut self.renderer
    }
}
