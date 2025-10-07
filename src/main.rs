use std::thread::sleep;
use std::time::Duration;
use crate::window::Chip8Window;

mod window;

fn main() {
    let mut window = Chip8Window::new();

    while window.should_run() {
        sleep(Duration::from_millis(10));
        window.update();
    }
}