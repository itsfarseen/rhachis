pub mod graphics;
pub mod input;
pub mod renderers;

use graphics::{Graphics, Renderer};
use input::Input;
use parking_lot::Mutex;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub use run_macro::run;

pub struct GameData {
    pub graphics: Mutex<Graphics>,
    pub input: Mutex<Input>,
    pub window: Mutex<Window>,
}

pub trait Game {
    fn init(data: &GameData) -> Self;
    fn update(&mut self, _: &GameData) {}
    fn get_renderer(&mut self) -> &mut dyn Renderer;
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
            window: Mutex::new(window),
        };

        let mut game = Self::init(&data);

        event_loop.run(move |event, _, control_flow| match event {
            Event::MainEventsCleared => {
                game.update(&data);
                game.get_renderer().update(&data);
                data.input.lock().update();
                data.window.lock().request_redraw();
            }
            Event::RedrawRequested(..) => data.graphics.lock().render(game.get_renderer()),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => data.input.lock().handle_key(input),
                WindowEvent::MouseInput { state, button, .. } => {
                    data.input.lock().handle_button(button, state)
                }
                WindowEvent::CursorMoved { position, .. } => {
                    data.input.lock().handle_cursor(position)
                }
                WindowEvent::Resized(size) => data.graphics.lock().resize(size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    data.graphics.lock().resize(*new_inner_size)
                }
                _ => {}
            },
            _ => {}
        });
    }
}
