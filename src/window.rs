use minifb::{Key, Scale, Window, WindowOptions};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub struct Chip8Window {
    window: Window,
}

impl Chip8Window {
    pub fn new() -> Self {
        let mut window = Window::new(
            "Iron Chip",
            WIDTH,
            HEIGHT,
            WindowOptions { scale: Scale::X16, ..Default::default() },
        )
        .unwrap();

        // Unrestrict this so the main game loop can handle setting FPS
        window.set_target_fps(0);

        Self { window }
    }

    pub fn should_run(&self) -> bool {
        self.window.is_open()
    }

    pub fn update(&mut self, buffer: &[u32; WIDTH * HEIGHT]) {
        self.window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }

    pub fn keyboard_state(&self) -> [bool; 16] {
        let keys_down: Vec<u8> = self.window.get_keys().iter().filter_map(|key| {
            match key {
                Key::Key1 => Some(0x1),
                Key::Key2 => Some(0x2),
                Key::Key3 => Some(0x3),
                Key::Key4 => Some(0xC),

                Key::Q => Some(0x4),
                Key::W => Some(0x5),
                Key::E => Some(0x6),
                Key::R => Some(0xD),

                Key::A => Some(0x7),
                Key::S => Some(0x8),
                Key::D => Some(0x9),
                Key::F => Some(0xE),

                Key::Z => Some(0xA),
                Key::X => Some(0x0),
                Key::C => Some(0xB),
                Key::V => Some(0xF),
                _ => None,
            }
        }).collect();

        let mut ret = [false; 16];
        for i in keys_down {
            ret[i as usize] = true;
        }

        ret
    }
}
