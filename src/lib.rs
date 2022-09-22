#![doc = include_str!("../README.md")]
pub mod graphics;
pub mod input;
pub mod math;
pub mod rand;
pub mod renderers;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use glam::UVec2;
use graphics::{Graphics, Renderer};
use input::Input;
use parking_lot::Mutex;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub use rhachis_run_macro::run;

/// A struct containing most of the global state of the engine.
pub struct GameData {
    /// Time since the last call of `Game::update`.
    pub delta_time: Duration,
    /// The time that the game started running.
    pub start_time: Instant,
    /// A handle to the graphics handler.
    pub graphics: Arc<Mutex<Graphics>>,
    /// A handle to the input handler.
    pub input: Arc<Mutex<Input>>,
    /// A handle to the winit window.
    pub window: Arc<Mutex<Window>>,
    /// A handle to the Exit code. It is recommended to use `GameData::exit` instead
    /// of directly modifying this value.
    pub exit_code: Arc<Mutex<Option<i32>>>,
}

impl GameData {
    /// Returns the size of the window being drawn to.
    pub fn get_window_size(&self) -> UVec2 {
        let size = self.window.lock().inner_size();
        UVec2::new(size.width, size.height)
    }

    /// Sets the size of the window being drawn to.
    pub fn set_window_size(&self, size: UVec2) {
        let size = PhysicalSize::new(size.x, size.y);
        self.window.lock().set_inner_size(size);
    }

    /// Quits the game upon next program update. If `code` is `Some`, then it is the
    /// exit code, otherwise the exit code is 0.
    pub fn exit(&self, code: Option<i32>) {
        *self.exit_code.lock() = Some(code.unwrap_or_default());
    }
}

#[allow(unused)]
/// A trait that all games must implement to use Rhachis
pub trait Game {
    /// Called when the game starts as a constructor for the initial state.
    fn init(data: &GameData) -> Self;
    /// Used to get the renderer when graphics need to be drawn.
    fn get_renderer(&mut self) -> &mut dyn Renderer;
    /// Called every update. Game logic should be handled here
    fn update(&mut self, data: &GameData) {}
    /// Called after every event is handled by the engine in case special behaviour
    /// is required for an event.
    fn handle_event(&mut self, data: &GameData, event: Event<()>) {}
}

/// Automatically implemented on everything that implements `Game`.
pub trait GameExt {
    /// Starts the game. This function never returns; code put after it will not
    /// be executed when the game quits.
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
        event_loop.run(move |event, _, control_flow| {
            match &event {
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
                    WindowEvent::KeyboardInput { input, .. } => {
                        data.input.lock().handle_key(*input)
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        data.input.lock().handle_button(*button, *state)
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        data.input.lock().handle_cursor(*position)
                    }
                    WindowEvent::Resized(size) => {
                        data.graphics.lock().resize(*size);
                        game.get_renderer().resize(&data);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        data.graphics.lock().resize(**new_inner_size);
                        game.get_renderer().resize(&data);
                    }
                    _ => {}
                },
                _ => {}
            }

            game.handle_event(&data, event)
        });
    }
}
