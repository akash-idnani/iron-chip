use minifb::{Scale, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Chip8Window {
    window: Window,
}

impl Chip8Window {
    pub fn new() -> Self {
        let mut window = Window::new(
            "Iron Chip",
            WIDTH,
            HEIGHT,
            WindowOptions {
                scale: Scale::X16,
                ..Default::default()
            }
        ).unwrap();

        // Unrestrict this so the main game loop can handle setting FPS
        window.set_target_fps(0);

        Self {
            window
        }

    }

    pub fn should_run(&self) -> bool {
        self.window.is_open()
    }

    pub fn update(&mut self, buffer: &[u32; WIDTH * HEIGHT]) {
        self.window
            .update_with_buffer(buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
