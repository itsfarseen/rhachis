use rhachis::*;

fn main() {
    Window::run();
}

struct Window;

impl Game for Window {
    fn init(_: &GameData) -> Self {
        Self
    }
}
