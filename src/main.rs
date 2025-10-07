use std::thread::sleep;
use std::time::Duration;
use crate::emulator::Chip8Emulator;
use crate::window::Chip8Window;

mod window;
mod emulator;

fn main() {
    let mut window = Chip8Window::new();
    let mut emulator = Chip8Emulator::new(Vec::new());

    let dummy = [0; 2048];

    while window.should_run() {
        sleep(Duration::from_millis(10));
        window.update(&dummy);
    }
}