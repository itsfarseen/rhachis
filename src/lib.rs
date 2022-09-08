pub mod graphics;
pub mod input;

use std::sync::Mutex;

use graphics::Graphics;
use input::Input;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct GameData {
    pub graphics: Mutex<Graphics>,
    pub input: Mutex<Input>,
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
            input: Mutex::new(Input::new()),
        };

        let mut game = Self::init(&data);

        event_loop.run(move |event, _, control_flow| match event {
            Event::MainEventsCleared => {
                game.update(&data);
                data.input.lock().unwrap().update();
                window.request_redraw();
            }
            Event::RedrawRequested(..) => data.graphics.lock().unwrap().render(),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    data.input.lock().unwrap().handle_key(input)
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    data.input.lock().unwrap().handle_button(button, state)
                }
                WindowEvent::CursorMoved { position, .. } => {
                    data.input.lock().unwrap().handle_cursor(position)
                }
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
