use crate::opcode::Opcode;
use std::fs;

const MEMORY_SIZE: usize = 4096;
const STACK_LEVELS: usize = 16;
const NUM_REGISTERS: usize = 16;
const NUM_KEYS: usize = 16;

const FONT_SET: [u8; 80] = [
	0xF0, 0x90, 0x90, 0x90, 0xF0,		// 0
	0x20, 0x60, 0x20, 0x20, 0x70,		// 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0,		// 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0,		// 3
	0x90, 0x90, 0xF0, 0x10, 0x10,		// 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0,		// 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0,		// 6
	0xF0, 0x10, 0x20, 0x40, 0x40,		// 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0,		// 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0,		// 9
	0xF0, 0x90, 0xF0, 0x90, 0x90,		// A
	0xE0, 0x90, 0xE0, 0x90, 0xE0,		// B
	0xF0, 0x80, 0x80, 0x80, 0xF0,		// C
	0xE0, 0x90, 0x90, 0x90, 0xE0,		// D
	0xF0, 0x80, 0xF0, 0x80, 0xF0,		// E
	0xF0, 0x80, 0xF0, 0x80, 0x80		// F
];

#[derive(Debug)]
pub struct Chip8 {
    memory_buffer: [u8; MEMORY_SIZE],
    stack: [u16; STACK_LEVELS],
    pc: usize, // Program Counter
    sp: usize, // Stack Pointer
    i: u16,    // Index Register
    v: [u8; NUM_REGISTERS],
    pub keys: [u8; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8,
    pub graphics: [[u8; 32]; 64],
}


impl Chip8 {
    pub fn new(filename: &str) -> Self {
        let memory_buffer = Chip8::load_file_into_memory(filename);
        Self {
            memory_buffer: memory_buffer,
            stack: [0; STACK_LEVELS],
            keys: [0; NUM_KEYS],
            v: [0; NUM_REGISTERS],
            graphics: [[0; 32]; 64],
            pc: 0x200,
            sp: 0,
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn load_file_into_memory(filename: &str) -> [u8; MEMORY_SIZE] {
        let mut memory_buffer: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        let contents =
            fs::read(filename).expect("Something went wrong when reading the CHIP-8 ROM");
        for item in contents.iter().enumerate() {
            let (idx, byte): (usize, &u8) = item;
            memory_buffer[idx + 512] = *byte;
        }

        for i in 0..80 {
            memory_buffer[i] = FONT_SET[i];
        }
        memory_buffer
    }

    pub fn emulate_cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.decode_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn fetch_opcode(&self) -> u16 {
        (self.memory_buffer[self.pc] as u16) << 8 | self.memory_buffer[self.pc + 1] as u16
    }

    fn clear_screen(&mut self, _opcode: &Opcode) {
        self.graphics = [[0; 32]; 64];
        self.pc += 2;
    }

    fn return_from_subroutine(&mut self, _opcode: &Opcode) {
        self.pc = self.stack[self.sp - 1] as usize;
        self.sp -= 1;
    }

    fn jump_to_nnn(&mut self, opcode: &Opcode) {
        let address = opcode.fetch_nnn();
        self.pc = address as usize;
    }

    fn call_subroutine_at_nnn(&mut self, opcode: &Opcode) {
        self.pc += 2;
        let address = opcode.fetch_nnn();
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc = address as usize;
    }

    fn skip_next_instruction_if_vx_equals_nn(&mut self, opcode: &Opcode) {
        let value = opcode.fetch_lowest_byte();
        let register_x_identifier = opcode.fetch_x();
        if self.v[register_x_identifier] == value {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn skip_next_instruction_if_vx_not_equals_nn(&mut self, opcode: &Opcode) {
        let value = opcode.fetch_lowest_byte();
        let register_x_identifier = opcode.fetch_x();
        if self.v[register_x_identifier] != value {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn skip_next_instruction_if_vx_equals_vy(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();
        if self.v[register_x_identifier] == self.v[register_y_identifier] {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn set_vx_to_nn(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let value = opcode.fetch_lowest_byte();
        self.v[register_x_identifier] = value;
        self.pc += 2;
    }

    fn add_nn_to_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let value = opcode.fetch_lowest_byte();
        self.v[register_x_identifier] = (self.v[register_x_identifier]).wrapping_add(value);
        self.pc += 2;
    }

    fn set_vx_to_vy(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();
        self.v[register_x_identifier] = self.v[register_y_identifier];
        self.pc += 2;
    }

    fn set_vx_to_vx_or_vy(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();
        self.v[register_x_identifier] =
            self.v[register_x_identifier] | self.v[register_y_identifier];
        self.pc += 2;
    }

    fn set_vx_to_vx_and_vy(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();
        self.v[register_x_identifier] =
            self.v[register_x_identifier] & self.v[register_y_identifier];
        self.pc += 2;
    }

    fn set_vx_to_vx_xor_vy(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();
        self.v[register_x_identifier] =
            self.v[register_x_identifier] ^ self.v[register_y_identifier];
        self.pc += 2;
    }

    fn add_vy_to_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();

        let (addition, overflow) = (self.v[register_x_identifier]).overflowing_add(self.v[register_y_identifier]);
        self.v[0xF] = if overflow {1} else {0};

        self.v[register_x_identifier] = addition;
        self.pc += 2;
    }

    fn subtract_vy_from_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();

        let (difference, overflow) = (self.v[register_x_identifier]).overflowing_sub(self.v[register_y_identifier]);

        self.v[0xF] = if !overflow {1} else {0};
        self.v[register_x_identifier] = difference;

        self.pc += 2;
    }

    fn store_least_significant_vx_bit_in_vf(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        self.v[0xF] = self.v[register_x_identifier] & 0x1;
        self.v[register_x_identifier] >>= 1;
        self.pc += 2;
    }

    fn set_vx_to_vy_minus_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();

        let (difference, overflow) = self.v[register_y_identifier].overflowing_sub(self.v[register_x_identifier]);

        self.v[0xF] = if !overflow {1} else {0};
        self.v[register_x_identifier] = difference;
        self.pc += 2;
    }

    fn store_most_significant_vx_bit_in_vf(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        self.v[0xF] = self.v[register_x_identifier] >> 7 & 0x1;
        self.v[register_x_identifier] <<=  1;
        self.pc += 2;
    }

    fn skip_next_instruction_if_vx_not_equals_vy(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();
        if self.v[register_x_identifier] != self.v[register_y_identifier] {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn set_i_to_nnn(&mut self, opcode: &Opcode) {
        let address = opcode.fetch_nnn();
        self.i = address;
        self.pc += 2;
    }

    fn jump_to_nnn_plus_v0(&mut self, opcode: &Opcode) {
        let address = opcode.fetch_nnn();
        self.pc = (self.v[0] as u16).wrapping_add(address) as usize;
    }

    fn set_vx_to_bitwise_and_with_rand(&mut self, opcode: &Opcode) {
        let value = opcode.fetch_lowest_byte();
        let register_x_identifier = opcode.fetch_x();
        self.v[register_x_identifier] = rand::random::<u8>() & value;
        self.pc += 2;
    }

    fn draw_sprite_at_vx_vy(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();
        let height = opcode.fetch_lowest_nibble();
        let register_x_value = self.v[register_x_identifier] % 64;
        let register_y_value = self.v[register_y_identifier] % 32;
        self.v[0xF] = 0;

        for height_offset in 0..height {
            let sprite_row = self.memory_buffer[(self.i + height_offset) as usize];

            for width_offset in 0..8 {
                if register_x_value + width_offset >= 64 {
                    continue
                }

                if register_y_value + height_offset as u8 >= 32 {
                    continue
                }

                let screen_pixel = self.graphics[(register_x_value + width_offset as u8) as usize][(register_y_value + height_offset as u8) as usize];
                let sprite_bit = (sprite_row >> (7 - width_offset)) & 0x1;

                // There is a collision, so set Vf.
                if screen_pixel == 1 && sprite_bit == 1 {
                    self.v[0xF] = 1;
                }

                self.graphics[(register_x_value + width_offset as u8) as usize][(register_y_value + height_offset as u8) as usize] ^= sprite_bit;
            }
        }

        self.pc += 2;
    }

    fn skip_next_instruction_if_vx_key_is_pressed(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        if self.keys[self.v[register_x_identifier] as usize] == 1 {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn skip_next_instruction_if_vx_key_is_not_pressed(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        if self.keys[self.v[register_x_identifier] as usize] == 0 {
            self.pc += 2;
        }
        self.pc += 2;
    }

    fn set_vx_to_delay_timer_value(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        self.v[register_x_identifier] = self.delay_timer;
        self.pc += 2;
    }

    fn await_key_press_and_store_in_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        for item in self.keys.iter().enumerate() {
            let (idx, value): (usize, &u8) = item;
            if *value == 1 {
                self.v[register_x_identifier] = idx as u8;
                self.pc += 2;
            }
        }
    }

    fn set_delay_timer_to_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        self.delay_timer = self.v[register_x_identifier];
        self.pc += 2;
    }

    fn set_sound_timer_to_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        self.sound_timer = self.v[register_x_identifier];
        self.pc += 2;
    }

    fn add_vx_to_i(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        self.i = (self.v[register_x_identifier] as u16).wrapping_add(self.i);
        self.pc += 2;
    }

    fn set_i_to_sprite_location(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        self.i = (self.v[register_x_identifier] as u16).wrapping_mul(5);
        self.pc += 2;
    }

    fn set_bcd_of_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_x_value = self.v[register_x_identifier];
        self.memory_buffer[self.i as usize] = register_x_value / 100;
        self.memory_buffer[(self.i + 1) as usize] = (register_x_value / 10) % 10;
        self.memory_buffer[(self.i + 2) as usize] = (register_x_value % 100) % 10;
        self.pc += 2;
    }

    fn register_dump(&mut self, opcode: &Opcode) {
        let x = opcode.fetch_x();
        for idx in 0..=x {
            let value = self.v[idx as usize];
            self.memory_buffer[self.i as usize + idx] = value;
        }
        self.pc += 2;
    }

    fn register_load(&mut self, opcode: &Opcode) {
        let x = opcode.fetch_x();
        for idx in 0..=x {
            self.v[idx as usize] = self.memory_buffer[self.i as usize + idx];
        }
        self.pc += 2;
    }

    fn decode_opcode(&mut self, opcode: u16) {
        let opcode = Opcode { value: opcode };
        let highest_nibble = opcode.fetch_highest_nibble();

        match highest_nibble {
            0x0000 => {
                let lowest_byte = opcode.fetch_lowest_byte();
                match lowest_byte {
                    0x00E0 => {
                        self.clear_screen(&opcode);
                    }
                    0x00EE => {
                        self.return_from_subroutine(&opcode);
                    }
                    _ => println!("No Match"),
                }
            }
            // 1NNN - Jumps to address NNN.
            0x1000 => {
                self.jump_to_nnn(&opcode);
            }
            // 2NNN - Calls subroutine at NNN.
            0x2000 => {
                self.call_subroutine_at_nnn(&opcode);
            }
            // 3XNN - Skip next instruction if Vx == NN
            0x3000 => {
                self.skip_next_instruction_if_vx_equals_nn(&opcode);
            }
            // 4XNN - Skip next instruction if Vx != NN
            0x4000 => {
                self.skip_next_instruction_if_vx_not_equals_nn(&opcode);
            }
            // 5XY0 - Skip next instruction if Vx == Vy
            0x5000 => {
                self.skip_next_instruction_if_vx_equals_vy(&opcode);
            }
            // 6XNN - Sets Vx to NN
            0x6000 => {
                self.set_vx_to_nn(&opcode);
            }
            // 7XNN - Adds N to Vx
            0x7000 => {
                self.add_nn_to_vx(&opcode);
            }
            0x8000 => {
                let lowest_nibble = opcode.fetch_lowest_nibble();
                match lowest_nibble {
                    // 8XY0 - Set Vx to Vy
                    0x0000 => {
                        self.set_vx_to_vy(&opcode);
                    }
                    // 8XY1 - Set Vx to Vx or Vy
                    0x0001 => {
                        self.set_vx_to_vx_or_vy(&opcode);
                    }
                    // 8XY2 - Set Vx to Vx & Vy
                    0x0002 => {
                        self.set_vx_to_vx_and_vy(&opcode);
                    }
                    // 8XY3 - Set Vx to Vx xor (^) Vy
                    0x0003 => {
                        self.set_vx_to_vx_xor_vy(&opcode);
                    }
                    // 8XY4 - Add Vy to Vx. VF is set to 0 when there's a borrow, and 1 when there
                    // is not.
                    0x0004 => {
                        self.add_vy_to_vx(&opcode);
                    }
                    // Vy is subtracted from Vx. Vf is set to 0 when there's a borrow and 1 when
                    // there is not.
                    0x0005 => {
                        self.subtract_vy_from_vx(&opcode);
                    }
                    // Stores the least significant bit of Vx in Vf and shits Vx to the right by 1.
                    0x0006 => {
                        self.store_least_significant_vx_bit_in_vf(&opcode);
                    }
                    // Sets Vx to Vy minus Vx. Vf is set to 0 when there's a borrow and 1 when
                    // there is not.
                    0x0007 => {
                        self.set_vx_to_vy_minus_vx(&opcode);
                    }
                    // Stores the most significant bit of Vx in Vf and shifts Vx to the left by 1.
                    0x000E => {
                        self.store_most_significant_vx_bit_in_vf(&opcode);
                    }
                    _ => println!("No Match"),
                }
            }
            // Skip the next instruction if Vx does not equal Vy.
            0x9000 => {
                self.skip_next_instruction_if_vx_not_equals_vy(&opcode);
            }
            // Sets I to the address NNN.
            0xA000 => {
                self.set_i_to_nnn(&opcode);
            }
            // Jumps to the address NNN plus V0.
            0xB000 => {
                self.jump_to_nnn_plus_v0(&opcode);
            }
            // Sets Vx to the result of a bitwise and operation on a random number (0-255).
            0xC000 => {
                self.set_vx_to_bitwise_and_with_rand(&opcode);
            }
            0xD000 => {
                self.draw_sprite_at_vx_vy(&opcode);
            }
            0xE000 => {
                let lowest_byte = opcode.fetch_lowest_byte();
                match lowest_byte {
                    0x009E => {
                        self.skip_next_instruction_if_vx_key_is_pressed(&opcode);
                    }
                    0x00A1 => {
                        self.skip_next_instruction_if_vx_key_is_not_pressed(&opcode);
                    }
                    _ => println!("No Match"),
                }
            }
            0xF000 => {
                let lowest_byte = opcode.fetch_lowest_byte();
                match lowest_byte {
                    0x0007 => {
                        self.set_vx_to_delay_timer_value(&opcode);
                    }
                    0x000A => {
                        self.await_key_press_and_store_in_vx(&opcode);
                    }
                    0x0015 => {
                        self.set_delay_timer_to_vx(&opcode);
                    }
                    0x0018 => {
                        self.set_sound_timer_to_vx(&opcode);
                    }
                    0x001E => {
                        self.add_vx_to_i(&opcode);
                    }
                    0x0029 => {
                        self.set_i_to_sprite_location(&opcode);
                    }
                    0x0033 => {
                        self.set_bcd_of_vx(&opcode);
                    }
                    0x0055 => {
                        self.register_dump(&opcode);
                    }
                    0x0065 => {
                        self.register_load(&opcode);
                    }
                    _ => println!("No Match"),
                }
            }
            _ => println!("No Match"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_ROM: &str = "roms/pong.ch8";

    #[test]
    fn test_jump_to_nnn() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.decode_opcode(0x1234);
        assert_eq!(chip8.pc, 0x0234);

        chip8.decode_opcode(0x1111);
        assert_eq!(chip8.pc, 0x0111);
    }

    #[test]
    #[should_panic]
    fn test_call_subroutine_at_nnn() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.decode_opcode(0x2234);
        assert_eq!(chip8.stack[0], 0x200);
        assert_eq!(chip8.sp, 1);
        assert_eq!(chip8.pc, 0x0234);

        for i in 1..16 {
            chip8.decode_opcode(0x2000 + i);
            assert_eq!(chip8.sp, (i + 1) as usize);
        }
        // We are out of stack space, and the rust kernel should panic.
        chip8.decode_opcode(0x2123);
    }

    #[test]
    fn test_skips_next_instruction_if_vx_equals_nn() {
        let mut chip8 = Chip8::new(TEST_ROM);
        let register_x_identifier = 0x2;

        chip8.v[register_x_identifier] = 0x0034;
        chip8.decode_opcode(0x3234);
        assert_eq!(chip8.pc, 0x200 + 4);

        chip8.decode_opcode(0x3212);
        // This instruction should not be skipped, since Vx is not equivalent to nn.
        assert_eq!(chip8.pc, 0x200 + 6);
    }

    #[test]
    fn test_skips_next_instruction_if_vx_not_equals_nn() {
        let mut chip8 = Chip8::new(TEST_ROM);
        let register_x_identifier = 0x2;

        chip8.v[register_x_identifier] = 0x0034;
        chip8.decode_opcode(0x4234);
        // This instruction should not be skipped, since Vx is equivalent to nn.
        assert_eq!(chip8.pc, 0x200 + 2);

        chip8.decode_opcode(0x4214);
        assert_eq!(chip8.pc, 0x200 + 6);
    }

    #[test]
    fn test_skips_next_instruction_if_vx_equals_vy() {
        let mut chip8 = Chip8::new(TEST_ROM);
        let register_x_identifier = 0x2;
        let register_y_identifier = 0x3;
        let value = 0x0034;

        chip8.v[register_x_identifier] = value;
        chip8.v[register_y_identifier] = value;
        chip8.decode_opcode(0x5230);
        // This instruction should be skipped, since Vx is equivalent to Vy.
        assert_eq!(chip8.pc, 0x200 + 4);

        chip8.decode_opcode(0x5260);
        assert_eq!(chip8.pc, 0x200 + 6);
    }

    #[test]
    fn test_set_vx_to_nn() {
        let mut chip8 = Chip8::new(TEST_ROM);
        chip8.decode_opcode(0x6234);

        assert_eq!(chip8.v[2], 0x0034);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_add_nn_to_vx() {
        let mut chip8 = Chip8::new(TEST_ROM);
        chip8.v[2] = 100;
        chip8.decode_opcode(0x7234);

        assert_eq!(chip8.v[2], 100 + 0x0034);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_set_vx_to_vy() {
        let mut chip8 = Chip8::new(TEST_ROM);
        chip8.v[2] = 100;
        chip8.v[3] = 200;
        chip8.decode_opcode(0x8230);

        assert_eq!(chip8.v[2], 200);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_set_vx_to_vx_or_vy() {
        let mut chip8 = Chip8::new(TEST_ROM);
        chip8.v[2] = 100;
        chip8.v[3] = 200;
        chip8.decode_opcode(0x8231);

        assert_eq!(chip8.v[2], 100 | 200);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_set_vx_to_vx_and_vy() {
        let mut chip8 = Chip8::new(TEST_ROM);
        chip8.v[2] = 100;
        chip8.v[3] = 200;
        chip8.decode_opcode(0x8232);

        assert_eq!(chip8.v[2], 100 & 200);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_set_vx_to_vx_xor_vy() {
        let mut chip8 = Chip8::new(TEST_ROM);
        chip8.v[2] = 100;
        chip8.v[3] = 200;
        chip8.decode_opcode(0x8233);

        assert_eq!(chip8.v[2], 100 ^ 200);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_add_vy_to_vx() {
        let mut chip8 = Chip8::new(TEST_ROM);
        chip8.v[2] = 50;
        chip8.v[3] = 60;
        chip8.decode_opcode(0x8234);

        assert_eq!(chip8.v[2], 110);
        assert_eq!(chip8.pc, 0x200 + 2);

        chip8.v[3] = 200;
        // Should overflow since value will be 110 + 200 >= 256.
        chip8.decode_opcode(0x8234);

        assert_eq!(chip8.v[15], 1);
        // Two's complement overflow.
        assert_eq!(chip8.v[2], 54);
    }

    #[test]
    fn test_subtract_vy_from_vx() {
        let mut chip8 = Chip8::new(TEST_ROM);
        chip8.v[2] = 3;
        chip8.v[3] = 1;
        chip8.decode_opcode(0x8235);

        assert_eq!(chip8.v[2], 2);
        assert_eq!(chip8.v[15], 1);
        assert_eq!(chip8.pc, 0x200 + 2);

        chip8.v[2] = 4;
        chip8.v[3] = 5;

        chip8.decode_opcode(0x8235);
        // There should be a borrow now.
        assert_eq!(chip8.v[15], 0);
    }

    #[test]
    fn test_least_significant_vx_bit_in_vf() {
        let mut chip8 = Chip8::new(TEST_ROM);
        chip8.v[2] = 3;

        chip8.decode_opcode(0x8236);

        assert_eq!(chip8.v[15], 1);
        // 3 >> 1 == 1
        assert_eq!(chip8.v[2], 1);
    }

    #[test]
    fn test_set_vx_to_vy_minus_vx() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.v[2] = 4;
        chip8.v[3] = 9;

        chip8.decode_opcode(0x8237);
        assert_eq!(chip8.v[2], 5);
        assert_eq!(chip8.v[15], 1);
        assert_eq!(chip8.pc, 0x200 + 2);

        chip8.v[2] = 12;
        chip8.v[3] = 4;
        chip8.decode_opcode(0x8237);
        assert_eq!(chip8.v[2], 248);
        assert_eq!(chip8.v[15], 0);
        assert_eq!(chip8.pc, 0x202 + 2);
    }

    #[test]
    fn test_store_most_significant_vx_bit_in_vf() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.v[2] = 255;

        chip8.decode_opcode(0x823E);

        assert_eq!(chip8.v[0xF], 1);
        assert_eq!(chip8.v[2], 254);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_not_equals_vy() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.v[2] = 20;
        chip8.v[3] = 20;

        chip8.decode_opcode(0x9230);

        assert_eq!(chip8.pc, 514);

        chip8.v[2] = 20;
        chip8.v[3] = 30;

        chip8.decode_opcode(0x9230);

        assert_eq!(chip8.pc, 518);
    }

    #[test]
    fn test_set_i_to_nnn() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.decode_opcode(0xA230);

        assert_eq!(chip8.i, 0x0230);
    }

    #[test]
    fn test_jump_to_nnn_plus_v0() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.v[0] = 0;

        chip8.decode_opcode(0xB230);

        assert_eq!(chip8.pc, 0x0230);

        chip8.v[0] = 5;

        chip8.decode_opcode(0xB230);

        assert_eq!(chip8.pc, 0x0235);
    }

    #[test]
    fn test_draw_sprite_at_vx_vy() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.v[0] = 0;
        chip8.v[1] = 0;
        chip8.i = 0x500;

        for width_offset in 0..8 {
            chip8.memory_buffer[(chip8.i + width_offset) as usize] = ((width_offset) % 2) as u8;
        }

        chip8.decode_opcode(0xD018);
        for x_coord in 0..8 {
            for y_coord in 0..8 {
                if x_coord == 7 && y_coord % 2 == 1 {
                    assert_eq!(chip8.graphics[x_coord][y_coord], 1);
                } else {
                    assert_eq!(chip8.graphics[x_coord][y_coord], 0);
                }
            }
        }

    }


    #[test]
    fn test_skip_next_instruction_if_vx_key_is_pressed() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.keys[0] = 1;

        chip8.v[0] = 0;

        chip8.decode_opcode(0xE09E);

        assert_eq!(chip8.pc, 0x200 + 4);

        chip8.keys[1] = 0;

        chip8.v[1] = 1;

        chip8.decode_opcode(0xE19E);

        assert_eq!(chip8.pc, 0x204 + 2);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_key_is_not_pressed() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.keys[0] = 1;

        chip8.v[0] = 0;

        chip8.decode_opcode(0xE0A1);

        assert_eq!(chip8.pc, 0x200 + 2);

        chip8.keys[1] = 0;

        chip8.v[1] = 1;

        chip8.decode_opcode(0xE1A1);

        assert_eq!(chip8.pc, 0x202 + 4);
    }

    #[test]
    fn test_set_vx_to_delay_timer_value() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.delay_timer = 15;

        chip8.decode_opcode(0xF007);

        assert_eq!(chip8.v[0], 15);
    }

    #[test]
    fn test_await_key_press_and_store_in_vx() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.decode_opcode(0xF00A);

        assert_eq!(chip8.pc, 0x200);

        chip8.keys[1] = 1;

        chip8.decode_opcode(0xF00A);

        assert_eq!(chip8.pc, 0x202);
        assert_eq!(chip8.v[0], 1);
    }

    #[test]
    fn test_set_delay_timer_to_vx() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.v[0] = 12;

        chip8.decode_opcode(0xF015);

        assert_eq!(chip8.delay_timer, 12);
    }

    #[test]
    fn test_set_sound_timer_to_vx() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.v[0] = 12;

        chip8.decode_opcode(0xF018);

        assert_eq!(chip8.sound_timer, 12);
    }

    #[test]
    fn test_add_vx_to_i() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.v[0] = 12;

        chip8.decode_opcode(0xF01E);

        assert_eq!(chip8.i, 12);
    }

    #[test]
    fn test_set_bcd_of_vx() {
        let mut chip8 = Chip8::new(TEST_ROM);

        chip8.v[0] = 243;

        chip8.decode_opcode(0xF033);

        assert_eq!(chip8.memory_buffer[chip8.i as usize], 2);
        assert_eq!(chip8.memory_buffer[(chip8.i + 1) as usize], 4);
        assert_eq!(chip8.memory_buffer[(chip8.i + 2) as usize], 3);
        assert_eq!(chip8.pc, 514);
    }

    #[test]
    fn test_register_dump() {
        let mut chip8 = Chip8::new(TEST_ROM);

        for i in 0..10 {
            chip8.v[i] = i as u8;
        }

        chip8.decode_opcode(0xFA55);

        for i in 0..10 {
            assert_eq!(chip8.memory_buffer[chip8.i as usize + i], i as u8);
        }
    }

    #[test]
    fn test_register_load() {
        let mut chip8 = Chip8::new(TEST_ROM);

        for i in 0..10 {
            chip8.memory_buffer[chip8.i as usize + i] = i as u8;
        }

        chip8.decode_opcode(0xFA65);

        for i in 0..10 {
            assert_eq!(chip8.v[i], i as u8);
        }
    }
}
