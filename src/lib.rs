pub trait Game {
    fn init() -> Self;
}

pub trait GameExt {
    fn run();
}

impl<T> GameExt for T
where T: Game {
    fn run() {
    }
}
