pub mod graphics;
pub mod input;
pub mod rand;
pub mod renderers;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

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
    pub delta_time: Duration,
    pub start_time: Instant,
    pub graphics: Arc<Mutex<Graphics>>,
    pub input: Arc<Mutex<Input>>,
    pub window: Arc<Mutex<Window>>,
    pub exit_code: Arc<Mutex<Option<i32>>>,
}

impl GameData {
    pub fn exit(&self, code: Option<i32>) {
        *self.exit_code.lock() = Some(code.unwrap_or_default());
    }
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

        let mut data = GameData {
            delta_time: Duration::ZERO,
            start_time: Instant::now(),
            graphics: Arc::new(Mutex::new(pollster::block_on(Graphics::new(&window)))),
            input: Arc::new(Mutex::new(Input::new())),
            window: Arc::new(Mutex::new(window)),
            exit_code: Arc::new(Mutex::new(None)),
        };

        let mut game = Self::init(&data);

        let mut last_update = Instant::now();
        event_loop.run(move |event, _, control_flow| match event {
            Event::MainEventsCleared => {
                data.delta_time = Instant::now() - last_update;

                game.update(&data);
                game.get_renderer().update(&data);
                if let Some(code) = *data.exit_code.lock() {
                    *control_flow = ControlFlow::ExitWithCode(code);
                }

                data.input.lock().update();
                data.window.lock().request_redraw();

                last_update = Instant::now();
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
                WindowEvent::Resized(size) => {
                    data.graphics.lock().resize(size);
                    game.get_renderer().resize(&data);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    data.graphics.lock().resize(*new_inner_size)
                }
                _ => {}
            },
            _ => {}
        });
    }
}
