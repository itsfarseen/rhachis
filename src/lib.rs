pub mod graphics;

use std::sync::Mutex;

use graphics::Graphics;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct GameData {
    graphics: Mutex<Graphics>,
}

pub trait Game {
    fn init(data: &GameData) -> Self;
    fn update(&mut self, _: &GameData) {}
}

pub trait GameExt {
    fn run();
}

impl<T> GameExt for T
where
    T: Game + 'static,
{
    fn run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let data = GameData {
            graphics: Mutex::new(pollster::block_on(Graphics::new(&window))),
        };

        let mut game = Self::init(&data);

        event_loop.run(move |event, _, control_flow| match event {
            Event::MainEventsCleared => {
                game.update(&data);
                window.request_redraw();
            }
            Event::RedrawRequested(..) => data.graphics.lock().unwrap().render(),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => data.graphics.lock().unwrap().resize(size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    data.graphics.lock().unwrap().resize(*new_inner_size)
                }
                _ => {}
            },
            _ => {}
        });
    }
}
