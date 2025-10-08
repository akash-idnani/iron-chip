use crate::window;
use crate::window::Chip8Window;

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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
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
    pub display_buffer: [u32; window::WIDTH * window::HEIGHT],

    instructions_per_frame: u8,
}

#[derive(Debug)]
struct DecodedInstruction {
    first_nibble: u8,
    x_register: u8, // Second nibble
    y_register: u8, // Third nibble
    n_4_bit_constant: u8, // Fourth nibble
    nn_8_bit_constant: u8, // Second byte
    nnn_12_bit_address: u16, // Second, third and fourth nibbles
    raw_instruction: u16,
}

impl Chip8Emulator {
    pub fn new(rom: Vec<u8>, instructions_per_frame: u8) -> Self {
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
            display_buffer: [0; window::WIDTH * window::HEIGHT],
            instructions_per_frame,
        }
    }

    pub fn run_60hz_frame(&mut self) {
        debug!("Running 60hz frame");
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
            debug!("Decrementing delay counter: {}", self.delay_timer);
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            debug!("Decrementing sound timer: {}", self.sound_timer);
        }

        for _ in 0..self.instructions_per_frame {
            self.run_instruction();
        }
    }

    fn run_instruction(&mut self) {
        let instruction = self.fetch();

        self.program_counter += 2;

        let decoded_instruction = Chip8Emulator::decode(instruction);
        match decoded_instruction {
            DecodedInstruction {raw_instruction: 0x00E0, ..} => { // Clear screen
                self.display_buffer.fill(0);
                debug!("Clearing display buffer");
            }

            _ => {
                error!("Unimplemented or invalid opcode {:?}", decoded_instruction);
            }
        }
    }

    fn fetch(&mut self) -> u16 {
        u16::from_be_bytes([
            self.ram[self.index_register as usize],
            self.ram[self.index_register as usize + 1],
        ])
    }

    fn decode(instruction: u16) -> DecodedInstruction {
        DecodedInstruction {
            first_nibble: (instruction >> 12) as u8,
            x_register: ((instruction >> 8) as u8) & 0xF,
            y_register: ((instruction >> 4) as u8) & 0xF,
            n_4_bit_constant: (instruction & 0xF) as u8,
            nn_8_bit_constant: instruction as u8,
            nnn_12_bit_address: instruction & 0x0FFF,
            raw_instruction: instruction,
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
        assert_eq!(
            emulator.ram[PROGRAM_START_ADDRESS as usize + PROGRAM_MAX_SIZE - 1],
            69
        );
    }

    #[test]
    #[should_panic(expected = "PROGRAM_MAX_SIZE")]
    fn test_emulator_too_large_rom_fails() {
        Chip8Emulator::new(vec![0; PROGRAM_MAX_SIZE + 1]);
    }
}
