use crate::opcode::Opcode;
use std::fs;

const MEMORY_SIZE: usize = 4096;
const STACK_LEVELS: usize = 16;
const NUM_REGISTERS: usize = 16;
const NUM_KEYS: usize = 16;

pub struct Chip8 {
    memory_buffer: [u8; MEMORY_SIZE],
    stack: [u16; STACK_LEVELS],
    pc: usize, // Program Counter
    sp: usize, // Stack Pointer
    I: u16,    // Index Register
    V: [u8; NUM_REGISTERS],
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
            V: [0; NUM_REGISTERS],
            graphics: [[0; 64]; 32],
            pc: 0x200,
            sp: 0,
            I: 0,
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

    fn decode_opcode(&mut self, opcode: u16) {
        let opcode = Opcode { value: opcode };
        let highest_nibble = opcode.fetch_highest_nibble();

        match highest_nibble {
            // 1NNN - Jumps to address NNN.
            0x1000 => {
                let address = opcode.fetch_nnn();
                self.pc = address as usize;
            }
            // 2NNN - Calls subroutine at NNN.
            0x2000 => {
                let address = opcode.fetch_nnn();
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                self.pc = address as usize;
            }
            // 3XNN - Skip next instruction if Vx == NN
            0x3000 => {
                let value = opcode.fetch_lowest_byte();
                let register_x_identifier = opcode.fetch_x();
                if self.V[register_x_identifier] == value {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // 4XNN - Skip next instruction if Vx != NN
            0x4000 => {
                let value = opcode.fetch_lowest_byte();
                let register_x_identifier = opcode.fetch_x();
                if self.V[register_x_identifier] != value {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // 5XY0 - Skip next instruction if Vx == Vy
            0x5000 => {
                let register_x_identifier = opcode.fetch_x();
                let register_y_identifier = opcode.fetch_y();
                if self.V[register_x_identifier] == self.V[register_y_identifier] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // 6XNN - Sets Vx to NN
            0x6000 => {
                let register_x_identifier = opcode.fetch_x();
                let value = opcode.fetch_lowest_byte();
                self.V[register_x_identifier] = value;
                self.pc += 2;
            }
            // 7XNN - Adds N to Vx
            0x7000 => {
                let register_x_identifier = opcode.fetch_x();
                let value = opcode.fetch_lowest_byte();
                self.V[register_x_identifier] += value;
                self.pc += 2;
            }
            0x8000 => {
                let lowest_nibble = opcode.fetch_lowest_nibble();
                match lowest_nibble {
                    // 8XY0 - Set Vx to Vy
                    0x0000 => {
                        let register_x_identifier = opcode.fetch_x();
                        let register_y_identifier = opcode.fetch_y();
                        self.V[register_x_identifier] = self.V[register_y_identifier];
                        self.pc += 2;
                    }
                    // 8XY1 - Set Vx to Vx or Vy
                    0x0001 => {
                        let register_x_identifier = opcode.fetch_x();
                        let register_y_identifier = opcode.fetch_y();
                        self.V[register_x_identifier] =
                            self.V[register_x_identifier] | self.V[register_y_identifier];
                        self.pc += 2;
                    }
                    // 8XY2 - Set Vx to Vx & Vy
                    0x0002 => {
                        let register_x_identifier = opcode.fetch_x();
                        let register_y_identifier = opcode.fetch_y();
                        self.V[register_x_identifier] =
                            self.V[register_x_identifier] & self.V[register_y_identifier];
                        self.pc += 2;
                    }
                    // 8XY3 - Set Vx to Vx xor (^) Vy
                    0x0003 => {
                        let register_x_identifier = opcode.fetch_x();
                        let register_y_identifier = opcode.fetch_y();
                        self.V[register_x_identifier] =
                            self.V[register_x_identifier] ^ self.V[register_y_identifier];
                        self.pc += 2;
                    }
                    // 8XY4 - Add Vy to Vx. VF is set to 0 when there's a borrow, and 1 when there
                    // is not.
                    0x0004 => {
                        let register_x_identifier = opcode.fetch_x();
                        let register_y_identifier = opcode.fetch_y();
                        let sum = self.V[register_x_identifier] + self.V[register_y_identifier];

                        if sum > 0xFF as u8 {
                            self.V[15] = 1;
                        }

                        self.V[register_x_identifier] = sum;
                        self.pc += 2;
                    }
                    // Vy is subtracted from Vx. Vf is set to 0 when there's a borrow and 1 when
                    // there is not.
                    0x0005 => {
                        let register_x_identifier = opcode.fetch_x();
                        let register_y_identifier = opcode.fetch_y();

                        let difference =
                            self.V[register_x_identifier] - self.V[register_y_identifier];

                        // TODO: VF is set to 0 when there's a borrow, and 1 when there is not.

                        self.V[register_x_identifier] = difference;
                        self.pc += 2;
                    }
                    // Stores the least significant bit of Vx in Vf and shits Vx to the right by 1.
                    0x0006 => {
                        let register_x_identifier = opcode.fetch_x();
                        self.V[15] = self.V[register_x_identifier] & 0b00000001;
                        self.V[register_x_identifier] = self.V[register_x_identifier] >> 1;
                        self.pc += 2;
                    }
                    // Sets Vx to Vy minus Vx. Vf is set to 0 when there's a borrow and 1 when
                    // there is not.
                    0x0007 => {
                        let register_x_identifier = opcode.fetch_x();
                        let register_y_identifier = opcode.fetch_y();

                        let difference =
                            self.V[register_y_identifier] - self.V[register_x_identifier];

                        // TODO: VF is set to 0 when there's a borrow, and 1 when there is not.

                        self.V[register_x_identifier] = difference;
                        self.pc += 2;
                    }
                    // Stores the most significant bit of Vx in Vf and shits Vx to the left by 1.
                    0x000E => {
                        let register_x_identifier = opcode.fetch_x();
                        self.V[15] = self.V[register_x_identifier] & 0b10000000;
                        self.V[register_x_identifier] = self.V[register_x_identifier] << 1;
                        self.pc += 2;
                    }
                    _ => println!("No Match"),
                }
            }
            // Skip the next instruction if Vx does not equal Vy.
            0x9000 => {
                let register_x_identifier = opcode.fetch_x();
                let register_y_identifier = opcode.fetch_y();
                if self.V[register_x_identifier] != self.V[register_y_identifier] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // Sets I to the address NNN.
            0xA000 => {
                let address = opcode.fetch_nnn();
                self.I = address;
                self.pc += 2;
            }
            // Jumps to the address NNN plus V0.
            0xB000 => {
                let address = opcode.fetch_nnn();
                self.pc = (self.V[0] as u16 + address) as usize;
            }
            // Sets Vx to the result of a bitwise and operation on a random number (0-255).
            0xC000 => {
                let value = opcode.fetch_lowest_byte();
                let register_x_identifier = opcode.fetch_x();
                self.V[register_x_identifier] = rand::random::<u8>() & value;
                self.pc += 2;
            }
            0xE000 => {
                let lowest_byte = opcode.fetch_lowest_byte();
                match lowest_byte {
                    0x009E => {
                        let register_x_identifier = opcode.fetch_x();
                        if self.keys[self.V[register_x_identifier] as usize] == 1 {
                            self.pc += 2;
                        }
                        self.pc += 2;
                    }
                    0x00A1 => {
                        let register_x_identifier = opcode.fetch_x();
                        if self.keys[self.V[register_x_identifier] as usize] == 0 {
                            self.pc += 2;
                        }
                        self.pc += 2;
                    }
                    _ => println!("No Match"),
                }
            }
            0xF000 => {
                let lowest_byte = opcode.fetch_lowest_byte();
                match lowest_byte {
                    0x0007 => {
                        let register_x_identifier = opcode.fetch_x();
                        self.V[register_x_identifier] = self.delay_timer;
                        self.pc += 2;
                    }
                    0x000A => {
                        let register_x_identifier = opcode.fetch_x();
                        for item in self.keys.iter().enumerate() {
                            let (idx, value): (usize, &u8) = item;
                            if *value == 1 {
                                self.V[register_x_identifier] = idx as u8;
                                self.pc += 2;
                            }
                        }
                    }
                    0x0015 => {
                        let register_x_identifier = opcode.fetch_x();
                        self.delay_timer = self.V[register_x_identifier];
                        self.pc += 2;
                    }
                    0x0018 => {
                        let register_x_identifier = opcode.fetch_x();
                        self.sound_timer = self.V[register_x_identifier];
                        self.pc += 2;
                    }
                    0x001E => {
                        let register_x_identifier = opcode.fetch_x();
                        self.I += self.V[register_x_identifier] as u16;
                        self.pc += 2;
                    }
                    0x0055 => {
                        let mut offset = 0;
                        let x = opcode.fetch_x();
                        for idx in 0..=x {
                            let value = self.V[idx as usize];
                            self.memory_buffer[(self.I + offset) as usize] = value;
                            offset += 1;
                        }
                        self.pc += 2;
                    }
                    0x0065 => {
                        let x = opcode.fetch_x();
                        let mut offset = 0;
                        for idx in 0..=x {
                            self.V[idx as usize] = self.memory_buffer[(self.I + offset) as usize];
                            offset += 1;
                        }
                        self.pc += 2;
                    }
                    _ => println!("No Match"),
                }
            }
            _ => println!("No Match"),
        }
    }
}
