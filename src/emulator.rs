const RAM_SIZE: usize = 4096;

/// First 0x200 bytes are reserved for the interpreter itself plus fonts
const PROGRAM_START_ADDRESS: u16 = 0x200;

const PROGRAM_MAX_SIZE: usize = RAM_SIZE - PROGRAM_START_ADDRESS as usize;

const FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8Emulator {
    registers: [u8; 16],
    ram: [u8; RAM_SIZE],
    index_register: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8Emulator {
    pub fn new(rom: Vec<u8>) -> Self {
        assert!(rom.len() <= PROGRAM_MAX_SIZE);

        let mut ram = [0; RAM_SIZE];

        // Place fonts into RAM starting at index 50
        for (index, font_byte) in FONTS.iter().enumerate() {
            ram[index + 0x50] = *font_byte;
        }

        // Place program into RAM
        for (index, program_byte) in rom.iter().enumerate() {
            ram[index + PROGRAM_START_ADDRESS as usize] = *program_byte;
        }

        Self {
            registers: Default::default(),
            ram,
            index_register: 0,
            program_counter: PROGRAM_START_ADDRESS,
            stack: Default::default(),
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_initial_ram() {
        let mut rom = vec![0; PROGRAM_MAX_SIZE];

        // Set some marker values at the beginning and end
        rom[0] = 42;
        rom[PROGRAM_MAX_SIZE - 1] = 69;

        let emulator = Chip8Emulator::new(rom);

        // Check some font values
        assert_eq!(emulator.ram[0x50], 0xF0);
        assert_eq!(emulator.ram[0x9F], 0x80);

        // Check the markers
        assert_eq!(emulator.ram[PROGRAM_START_ADDRESS as usize], 42);
        assert_eq!(emulator.ram[PROGRAM_START_ADDRESS as usize + PROGRAM_MAX_SIZE - 1], 69);
    }

    #[test]
    #[should_panic(expected = "PROGRAM_MAX_SIZE")]
    fn test_emulator_too_large_rom_fails() {
        Chip8Emulator::new(
            vec![0; PROGRAM_MAX_SIZE + 1]
        );
    }
}

