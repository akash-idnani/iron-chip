use crate::window;
use crate::window::WIDTH;

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
    x_register: u8,          // Second nibble
    y_register: u8,          // Third nibble
    n_4_bit_constant: u8,    // Fourth nibble
    nn_8_bit_constant: u8,   // Second byte
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
            //00E0: Clears the screen
            DecodedInstruction {
                raw_instruction: 0x00E0,
                ..
            } => {
                self.display_buffer.fill(0);
                debug!("0x00E0: Clearing display buffer");
            }

            // 1NNN: Jump to address NNN
            DecodedInstruction {
                first_nibble: 0x1,
                nnn_12_bit_address,
                raw_instruction,
                ..
            } => {
                self.program_counter = nnn_12_bit_address;
                debug!("{raw_instruction:#X}: Jumping to address {nnn_12_bit_address:#3X}");
            }

            // 3XNN: Skips the next instruction if VX equals NN
            // (usually the next instruction is a jump to skip a code block).
            DecodedInstruction {
                first_nibble: 0x3,
                x_register,
                nn_8_bit_constant,
                raw_instruction,
                ..
            } => {
                if self.registers[x_register as usize] == nn_8_bit_constant {
                    self.program_counter += 2;
                    debug!("{raw_instruction:#X}: Skipping because register {x_register} == {nn_8_bit_constant}");
                } else {
                    debug!("{raw_instruction:#X}: Not skipping because register {x_register} != {nn_8_bit_constant}");
                }
            }

            // 6XNN: Sets VX to NN
            DecodedInstruction {
                first_nibble: 0x6,
                x_register,
                nn_8_bit_constant,
                raw_instruction,
                ..
            } => {
                self.registers[x_register as usize] = nn_8_bit_constant;
                debug!("{raw_instruction:#X}: Setting register {x_register} to {nn_8_bit_constant:#2X}");
            }

            // 7XNN: Adds NN to VX (carry flag is not changed)
            DecodedInstruction {
                first_nibble: 0x7,
                x_register,
                nn_8_bit_constant,
                raw_instruction,
                ..
            } => {
                self.registers[x_register as usize] += nn_8_bit_constant;
                debug!("{raw_instruction:#X}: Adding {nn_8_bit_constant} to register {x_register}");
            }

            // AXNN: Sets I to the address NNN.
            DecodedInstruction {
                first_nibble: 0xA,
                nnn_12_bit_address,
                raw_instruction,
                ..
            } => {
                self.index_register = nnn_12_bit_address;
                debug!("{raw_instruction:#X}: Setting index register to {nnn_12_bit_address:#3X}");
            }

            // DXYN:
            // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
            // Each row of 8 pixels is read as bit-coded starting from memory location I;
            // I value does not change after the execution of this instruction.
            // As described above, VF is set to 1 if any screen pixels are flipped from set
            // to unset when the sprite is drawn, and to 0 if that does not happen
            DecodedInstruction {
                first_nibble: 0xD,
                x_register,
                y_register,
                n_4_bit_constant,
                raw_instruction,
                ..
            } => {
                let x = self.registers[x_register as usize] as usize;
                let y = self.registers[y_register as usize] as usize;
                let height = n_4_bit_constant as usize;

                let mut collision_detected = false;

                for y_counter in 0..height {
                    for x_counter in 0..8 {
                        let is_pixel_on = (self.ram[self.index_register as usize + y_counter]
                            & (0x80 >> x_counter))
                            != 0;

                        let dest_address = (y_counter + y) * WIDTH + (x_counter + x);
                        let is_already_on = self.display_buffer[dest_address] != 0;

                        if is_pixel_on && is_already_on {
                            collision_detected = true;
                        }

                        if is_pixel_on {
                            self.display_buffer[dest_address] = 0xFFFFFFFF;
                        }
                    }
                }

                if collision_detected {
                    self.registers[0xF] = 1;
                }

                debug!("{raw_instruction:#X}: Drawing sprite at address {:#3X} of height {height} to ({x}, {y}). Collision Detected: {collision_detected}",
                    self.index_register);
            }

            _ => {
                error!(
                    "Unimplemented or invalid opcode {:#X}",
                    decoded_instruction.raw_instruction
                );
            }
        }
    }

    fn fetch(&mut self) -> u16 {
        u16::from_be_bytes([
            self.ram[self.program_counter as usize],
            self.ram[self.program_counter as usize + 1],
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

    fn assert_pixel(emulator: &Chip8Emulator, display_buffer_addr: usize, set: bool) {
        if set {
            assert_ne!(emulator.display_buffer[display_buffer_addr], 0);
        } else {
            assert_eq!(emulator.display_buffer[display_buffer_addr], 0);
        }
    }

    #[test]
    fn test_initial_ram() {
        let mut rom = vec![0; PROGRAM_MAX_SIZE];

        // Set some marker values at the beginning and end
        rom[0] = 42;
        rom[PROGRAM_MAX_SIZE - 1] = 69;

        let emulator = Chip8Emulator::new(rom, 10);

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
        Chip8Emulator::new(vec![0; PROGRAM_MAX_SIZE + 1], 10);
    }

    #[test]
    fn test_00e0() {
        let mut emulator = Chip8Emulator::new(vec![0x00, 0xE0], 10);

        emulator.display_buffer.fill(69);
        emulator.run_instruction();

        assert!(emulator.display_buffer.iter().all(|i| *i == 0));
    }

    #[test]
    fn test_1nnn() {
        let mut emulator = Chip8Emulator::new(vec![0x12, 0x34], 10);
        emulator.run_instruction();
        assert_eq!(emulator.program_counter, 0x234);
    }

    #[test]
    fn test_3xnn() {
        let program = vec![
            0x61, 0x12, // Set register 1 to 0x12
            0x31, 0x00, // If register 1 == 0, skip next instruction
            0x31, 0x12, // If register 1 == 0x12, skip next instruction
        ];

        let mut emulator = Chip8Emulator::new(program, 10);
        emulator.run_instruction();
        emulator.run_instruction(); // Should not skip
        assert_eq!(emulator.program_counter, PROGRAM_START_ADDRESS + 4);

        emulator.run_instruction(); // Should skip
        assert_eq!(emulator.program_counter, PROGRAM_START_ADDRESS + 8);
    }

    #[test]
    fn test_6xnn() {
        let mut emulator = Chip8Emulator::new(vec![0x60, 0x12, 0x6e, 0x34], 10);

        emulator.run_instruction();
        assert_eq!(emulator.registers[0], 0x12);

        emulator.run_instruction();
        assert_eq!(emulator.registers[0xe], 0x34);
    }

    #[test]
    fn test_7xnn() {
        let mut emulator = Chip8Emulator::new(vec![0x71, 0x01, 0x71, 0x02], 10);

        emulator.run_instruction();
        assert_eq!(emulator.registers[1], 0x1);

        emulator.run_instruction();
        assert_eq!(emulator.registers[1], 0x3);
    }

    #[test]
    fn test_dxyn() {
        let program: Vec<u8> = vec![
            0x60, 1, // Set register 0 to 1
            0x61, 2, // Set register 1 to 2
            0x62, 3, // Set register 2 to 3
            0xA2, 0x0C, // Set index register to 0x20C
            0xD0, 0x12, // Display to location (1, 2), height 2
            0xD0, 0x22,       // Display to location (1, 3), height 2
            0xFF,       // Bitmask row 1
            0b10101010, // Bitmask row 2
        ];

        let mut emulator = Chip8Emulator::new(program, 10);
        for _ in 0..5 {
            emulator.run_instruction();
        }

        assert_eq!(emulator.registers[0xF], 0); // no collision

        // First row, everything should be set
        assert_pixel(&emulator, 2 * WIDTH, false);
        for i in 1..=8 {
            assert_pixel(&emulator, 2 * WIDTH + i, true);
        }
        assert_pixel(&emulator, 2 * WIDTH + 9, false);

        // Second row, alternating
        assert_pixel(&emulator, 3 * WIDTH, false);
        for i in 1..=8 {
            assert_pixel(&emulator, 3 * WIDTH + i, i % 2 == 1);
        }
        assert_pixel(&emulator, 3 * WIDTH + 9, false);

        // Should overwrite row 3, leave row 2
        emulator.run_instruction();

        assert_eq!(emulator.registers[0xF], 1); // Collision

        // First row, everything should be set
        assert_pixel(&emulator, 2 * WIDTH, false);
        for i in 1..=8 {
            assert_pixel(&emulator, 2 * WIDTH + i, true);
        }
        assert_pixel(&emulator, 2 * WIDTH + 9, false);

        // Second row, everything should be set
        assert_pixel(&emulator, 3 * WIDTH, false);
        for i in 1..=8 {
            assert_pixel(&emulator, 3 * WIDTH + i, true);
        }
        assert_pixel(&emulator, 3 * WIDTH + 9, false);

        // New third row, alternating
        assert_pixel(&emulator, 4 * WIDTH, false);
        for i in 1..=8 {
            assert_pixel(&emulator, 4 * WIDTH + i, i % 2 == 1);
        }
        assert_pixel(&emulator, 4 * WIDTH + 9, false);
    }
}
