use crate::emulator::Chip8Emulator;
use crate::window::Chip8Window;
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};

mod emulator;
mod window;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, value_name = "FILE")]
    rom_file: PathBuf,
}

fn main() {
    pretty_env_logger::init();
    info!("Starting Emulator");

    let args = Args::parse();
    let rom_data = fs::read(args.rom_file).expect("Couldn't read ROM");

    let mut window = Chip8Window::new();
    let mut emulator = Chip8Emulator::new(rom_data, 12);

    const INTERVAL: Duration = Duration::from_micros(16667); // 60Hz

    while window.should_run() {
        let frame_start_time = Instant::now();

        emulator.run_60hz_frame(window.keyboard_state());
        window.update(&emulator.display_buffer);

        let current_runtime = Instant::now().duration_since(frame_start_time);

        if current_runtime >= INTERVAL {
            warn!("WARNING: Exceeded 60Hz Frame! Runtime: {:?}", current_runtime);
        } else {
            sleep(INTERVAL - current_runtime);
        }
    }
}
