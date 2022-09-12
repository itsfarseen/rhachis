use std::f32::consts::TAU;

use glam::Mat4;
use rhachis::{
    renderers::{Model, SimpleRenderer},
    Game, GameExt,
};

fn main() {
    Obj::run();
}

struct Obj {
    renderer: SimpleRenderer,
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
            Model::from_obj(data, "examples/test.obj")
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
