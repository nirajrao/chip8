use crate::opcode::Opcode;
use std::fs;
use std::num::Wrapping;

const MEMORY_SIZE: usize = 4096;
const STACK_LEVELS: usize = 16;
const NUM_REGISTERS: usize = 16;
const NUM_KEYS: usize = 16;

#[derive(Debug)]
pub struct Chip8 {
    memory_buffer: [u8; MEMORY_SIZE],
    stack: [u16; STACK_LEVELS],
    pc: usize, // Program Counter
    sp: usize, // Stack Pointer
    i: u16,    // Index Register
    v: [u8; NUM_REGISTERS],
    keys: [u8; NUM_KEYS],
    delay_timer: u8,
    sound_timer: u8,
    graphics: [[u8; 64]; 32],
}

impl Chip8 {
    pub fn new(filename: &str) -> Self {
        let memory_buffer = Chip8::load_file_into_memory(filename);
        Self {
            memory_buffer: memory_buffer,
            stack: [0; STACK_LEVELS],
            keys: [0; NUM_KEYS],
            v: [0; NUM_REGISTERS],
            graphics: [[0; 64]; 32],
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
        memory_buffer
    }

    fn fetch_opcode(&self) -> u16 {
        (self.memory_buffer[self.pc] as u16) << 8 | self.memory_buffer[self.pc + 1] as u16
    }

    fn jump_to_nnn(&mut self, opcode: &Opcode) {
        let address = opcode.fetch_nnn();
        self.pc = address as usize;
    }

    fn call_subroutine_at_nnn(&mut self, opcode: &Opcode) {
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
        self.v[register_x_identifier] += value;
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

        if self.v[register_y_identifier] > 0xFF - self.v[register_x_identifier] {
            self.v[15] = 1;
        } else {
            self.v[15] = 0;
        }

        self.v[register_x_identifier] =
            (Wrapping(self.v[register_x_identifier]) + Wrapping(self.v[register_y_identifier])).0;
        self.pc += 2;
    }

    fn subtract_vy_from_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();

        let mut mask = 0x1;
        let mut borrow = false;
        while mask < 0x80 {
            if self.v[register_y_identifier] & mask > self.v[register_x_identifier] & mask {
                self.v[15] = 0;
                borrow = true;
            }
            mask <<= 1;
        }
        if !borrow {
            self.v[15] = 1;
        }

        self.v[register_x_identifier] =
            (Wrapping(self.v[register_x_identifier]) - Wrapping(self.v[register_y_identifier])).0;
        self.pc += 2;
    }

    fn store_least_significant_vx_bit_in_vf(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        self.v[15] = self.v[register_x_identifier] & 0b00000001;
        self.v[register_x_identifier] = self.v[register_x_identifier] >> 1;
        self.pc += 2;
    }

    fn set_vx_to_vy_minus_vx(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        let register_y_identifier = opcode.fetch_y();

        let difference = self.v[register_y_identifier] - self.v[register_x_identifier];

        // TODO: VF is set to 0 when there's a borrow, and 1 when there is not.

        self.v[register_x_identifier] = difference;
        self.pc += 2;
    }

    fn store_most_significant_vx_bit_in_vf(&mut self, opcode: &Opcode) {
        let register_x_identifier = opcode.fetch_x();
        self.v[15] = self.v[register_x_identifier] & 0b10000000;
        self.v[register_x_identifier] = self.v[register_x_identifier] << 1;
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
        self.pc = (self.v[0] as u16 + address) as usize;
    }

    fn set_vx_to_bitwise_and_with_rand(&mut self, opcode: &Opcode) {
        let value = opcode.fetch_lowest_byte();
        let register_x_identifier = opcode.fetch_x();
        self.v[register_x_identifier] = rand::random::<u8>() & value;
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
        self.i += self.v[register_x_identifier] as u16;
        self.pc += 2;
    }

    fn register_dump(&mut self, opcode: &Opcode) {
        let mut offset = 0;
        let x = opcode.fetch_x();
        for idx in 0..=x {
            let value = self.v[idx as usize];
            self.memory_buffer[(self.i + offset) as usize] = value;
            offset += 1;
        }
        self.pc += 2;
    }

    fn register_load(&mut self, opcode: &Opcode) {
        let x = opcode.fetch_x();
        let mut offset = 0;
        for idx in 0..=x {
            self.v[idx as usize] = self.memory_buffer[(self.i + offset) as usize];
            offset += 1;
        }
        self.pc += 2;
    }

    fn decode_opcode(&mut self, opcode: u16) {
        let opcode = Opcode { value: opcode };
        let highest_nibble = opcode.fetch_highest_nibble();

        match highest_nibble {
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

    #[test]
    fn test_jump_to_nnn() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let first_opcode = Opcode { value: 0x1234 };
        let second_opcode = Opcode { value: 0x1111 };

        // Even though `jump_to_nnn` takes a mutable **reference** to a Chip8 instance, Rust knows
        // how to automatically reference the instance.
        chip8.jump_to_nnn(&first_opcode);
        assert_eq!(chip8.pc, 0x0234);

        chip8.jump_to_nnn(&second_opcode);
        assert_eq!(chip8.pc, 0x0111);
    }

    #[test]
    #[should_panic]
    fn test_call_subroutine_at_nnn() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };

        chip8.call_subroutine_at_nnn(&opcode);
        assert_eq!(chip8.stack[0], 0x200);
        assert_eq!(chip8.sp, 1);
        assert_eq!(chip8.pc, 0x0234);

        for i in 1..16 {
            let opcode = Opcode { value: i };
            chip8.call_subroutine_at_nnn(&opcode);
            assert_eq!(chip8.sp, (i + 1) as usize);
        }
        // We are out of stack space, and the rust kernel should panic.
        chip8.call_subroutine_at_nnn(&opcode);
    }

    #[test]
    fn test_skips_next_instruction_if_vx_equals_nn() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        let register_x_identifier = 0x2;

        chip8.v[register_x_identifier] = 0x0034;
        chip8.skip_next_instruction_if_vx_equals_nn(&opcode);
        assert_eq!(chip8.pc, 0x200 + 4);

        let second_opcode = Opcode { value: 0x4567 };
        chip8.skip_next_instruction_if_vx_equals_nn(&second_opcode);
        // This instruction should not be skipped, since Vx is not equivalent to nn.
        assert_eq!(chip8.pc, 0x200 + 6);
    }

    #[test]
    fn test_skips_next_instruction_if_vx_not_equals_nn() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        let register_x_identifier = 0x2;

        chip8.v[register_x_identifier] = 0x0034;
        chip8.skip_next_instruction_if_vx_not_equals_nn(&opcode);
        // This instruction should not be skipped, since Vx is equivalent to nn.
        assert_eq!(chip8.pc, 0x200 + 2);

        let second_opcode = Opcode { value: 0x4567 };
        chip8.skip_next_instruction_if_vx_not_equals_nn(&second_opcode);
        assert_eq!(chip8.pc, 0x200 + 6);
    }

    #[test]
    fn test_skips_next_instruction_if_vx_equals_vy() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        let register_x_identifier = 0x2;
        let register_y_identifier = 0x3;
        let value = 0x0034;

        chip8.v[register_x_identifier] = value;
        chip8.v[register_y_identifier] = value;
        chip8.skip_next_instruction_if_vx_equals_vy(&opcode);
        // This instruction should not be skipped, since Vx is equivalent to nn.
        assert_eq!(chip8.pc, 0x200 + 4);

        let second_opcode = Opcode { value: 0x4267 };
        chip8.skip_next_instruction_if_vx_equals_vy(&second_opcode);
        assert_eq!(chip8.pc, 0x200 + 6);
    }

    #[test]
    fn test_set_vx_to_nn() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        chip8.set_vx_to_nn(&opcode);

        assert_eq!(chip8.v[2], 0x0034);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_add_nn_to_vx() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        chip8.v[2] = 100;
        chip8.add_nn_to_vx(&opcode);

        assert_eq!(chip8.v[2], 100 + 0x0034);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_set_vx_to_vy() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        chip8.v[2] = 100;
        chip8.v[3] = 200;
        chip8.set_vx_to_vy(&opcode);

        assert_eq!(chip8.v[2], 200);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_set_vx_to_vx_or_vy() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        chip8.v[2] = 100;
        chip8.v[3] = 200;
        chip8.set_vx_to_vx_or_vy(&opcode);

        assert_eq!(chip8.v[2], 100 | 200);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_set_vx_to_vx_and_vy() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        chip8.v[2] = 100;
        chip8.v[3] = 200;
        chip8.set_vx_to_vx_and_vy(&opcode);

        assert_eq!(chip8.v[2], 100 & 200);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_set_vx_to_vx_xor_vy() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        chip8.v[2] = 100;
        chip8.v[3] = 200;
        chip8.set_vx_to_vx_xor_vy(&opcode);

        assert_eq!(chip8.v[2], 100 ^ 200);
        assert_eq!(chip8.pc, 0x200 + 2);
    }

    #[test]
    fn test_add_vy_to_vx() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        chip8.v[2] = 50;
        chip8.v[3] = 60;
        chip8.add_vy_to_vx(&opcode);

        assert_eq!(chip8.v[2], 110);
        assert_eq!(chip8.pc, 0x200 + 2);

        chip8.v[3] = 200;
        // Should overflow since value will be 110 + 200 >= 256.
        chip8.add_vy_to_vx(&opcode);

        assert_eq!(chip8.v[15], 1);
        // Two's complement overflow.
        assert_eq!(chip8.v[2], 54);
    }

    #[test]
    fn test_subtract_vy_from_vx() {
        let mut chip8 = Chip8::new("roms/pong.ch8");
        let opcode = Opcode { value: 0x1234 };
        chip8.v[2] = 3;
        chip8.v[3] = 1;
        chip8.subtract_vy_from_vx(&opcode);

        assert_eq!(chip8.v[2], 2);
        assert_eq!(chip8.v[15], 1);
        assert_eq!(chip8.pc, 0x200 + 2);

        chip8.v[2] = 4;
        chip8.v[3] = 2;
        chip8.subtract_vy_from_vx(&opcode);

        // There should be a borrow now.
        assert_eq!(chip8.v[15], 0);
    }
}
