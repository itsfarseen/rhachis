use winit::{event_loop::{EventLoop, ControlFlow}, window::WindowBuilder, event::{Event, WindowEvent}};

pub trait Game {
    fn init() -> Self;
}

pub trait GameExt {
    fn run();
}

impl<T> GameExt for T
where T: Game {
    fn run() {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {},
            }
            _ => {}
        });
    }
}
