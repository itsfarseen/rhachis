use rhachis::*;

fn main() {
    Window::run();
}

struct Window;

impl Game for Window {
    fn init() -> Self {
        Self
    }
}
