use std::fs;

const MEMORY_SIZE: usize = 4096;
const STACK_LEVELS: usize = 16;
const NUM_REGISTERS: usize = 16;
const SCREEN_SIZE: u32 = 64 * 32;

pub struct Chip8 {
    memory_buffer: [u8; MEMORY_SIZE],
    stack: [u32; STACK_LEVELS],
    pc: usize, // Program Counter
    sp: usize, // Stack Pointer
    registers: [u16; NUM_REGISTERS],
}

impl Chip8 {
    pub fn new(filename: &str) -> Self {
        let memory_buffer = Chip8::load_file_into_memory(filename);
        Self {
            memory_buffer: memory_buffer,
            stack: [0; STACK_LEVELS],
            registers: [0; NUM_REGISTERS],
            pc: 0x200,
            sp: 0,
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

    fn fetch_opcode(&mut self) -> u16 {
        (self.memory_buffer[self.pc] as u16) << 8 | self.memory_buffer[self.pc + 1] as u16
    }

    fn decode_opcode(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            // 1NNN - Jumps to address NNN.
            0x1000 => {
                let address = opcode & 0x0FFF;
                self.pc = address as usize;
            }
            // 2NNN - Calls subroutine at NNN.
            0x2000 => {
                let address = opcode & 0x0FFF;
                self.stack[self.sp] = self.pc as u32;
                self.sp += 1;
                self.pc = opcode as usize;
            }
            // 3XNN - Skip next instruction if Vx == NN
            0x3000 => {
                let value = opcode & 0x00FF;
                let register_number = opcode & 0x0F00;
                if self.registers[register_number as usize] == value {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // 4XNN - Skip next instruction if Vx != NN
            0x4000 => {
                let value = opcode & 0x00FF;
                let register_number = opcode & 0x0F00;
                if self.registers[register_number as usize] != value {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // 5XY0 - Skip next instruction if Vx == Vy
            0x5000 => {
                let register_x = opcode & 0x0F00;
                let register_y = opcode & 0x00F0;
                if self.registers[register_x as usize] == self.registers[register_y as usize] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            // 6XNN - Sets Vx to NN
            0x6000 => {
                let register_x = opcode & 0x0F00;
                let value = opcode & 0x00FF;
                self.registers[register_x as usize] = value;
                self.pc += 2;
            }
            // 7XNN - Adds N to Vx
            0x7000 => {
                let register_x = opcode & 0x0F00;
                let value = opcode & 0x00FF;
                self.registers[register_x as usize] += value;
                self.pc += 2;
            }
            0x8000 => {
                match opcode & 0x000F {
                    // 8XY0 - Set Vx to Vy
                    0x0000 => {
                        let register_x = opcode & 0x0F00;
                        let register_y = opcode & 0x00F0;
                        self.registers[register_x as usize] = self.registers[register_y as usize];
                        self.pc += 2;
                    }
                    // 8XY1 - Set Vx to Vx or Vy
                    0x001 => {
                        let register_x = opcode & 0x0F00;
                        let register_y = opcode & 0x00F0;
                        self.registers[register_x as usize] = self.registers[register_x as usize]
                            | self.registers[register_y as usize];
                        self.pc += 2;
                    }
                    // 8XY2 - Set Vx to Vx & Vy
                    0x002 => {
                        let register_x = opcode & 0x0F00;
                        let register_y = opcode & 0x00F0;
                        self.registers[register_x as usize] = self.registers[register_x as usize]
                            & self.registers[register_y as usize];
                        self.pc += 2;
                    }
                    // 8XY3 - Set Vx to Vx xor (^) Vy
                    0x003 => {
                        let register_x = opcode & 0x0F00;
                        let register_y = opcode & 0x00F0;
                        self.registers[register_x as usize] = self.registers[register_x as usize]
                            ^ self.registers[register_y as usize];
                        self.pc += 2;
                    }
                    // 8XY4 - Add Vy to Vx. VF is set to 0 when there's a borrow, and 1 when there
                    // is not.
                    0x004 => {
                        let register_x = opcode & 0x0F00;
                        let register_y = opcode & 0x00F0;
                        let sum = self.registers[register_x as usize]
                            + self.registers[register_y as usize];

                        if sum > 0xFF {
                            self.registers[15] = 1;
                        }

                        self.registers[register_x as usize] = sum;
                        self.pc += 2;
                    }
                }
            }

            _ => println!("No Match"),
        }
    }
}
