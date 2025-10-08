use crate::emulator::Chip8Emulator;
use crate::window::Chip8Window;
use std::thread::sleep;
use std::time::{Duration, Instant};

mod emulator;
mod window;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

fn main() {
    pretty_env_logger::init();
    info!("Starting Emulator");

    let mut window = Chip8Window::new();
    let mut emulator = Chip8Emulator::new(Vec::new(), 12);

    const INTERVAL: Duration = Duration::from_micros(16667); // 60Hz

    while window.should_run() {
        let frame_start_time = Instant::now();

        emulator.run_60hz_frame();
        window.update(&emulator.display_buffer);

        let current_runtime = Instant::now().duration_since(frame_start_time);

        if current_runtime >= INTERVAL  {
            trace!("WARNING: Exceeded 60Hz Frame! Runtime: {:?}", current_runtime);
        } else {
            sleep(INTERVAL - current_runtime);
        }
    }
}
