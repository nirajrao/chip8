use std::fs;

const MEMORY_SIZE: usize = 4096;
const NUM_REGISTERS: u32 = 16;
const SCREEN_SIZE: u32 = 64 * 32;
const STACK_LEVELS: u32 = 16;

pub struct Chip8 {
    memory_buffer: [u8; MEMORY_SIZE],
}

impl Chip8 {
    pub fn new(filename: &str) -> Self {
        let memory_buffer = Chip8::load_file_into_memory(filename);
        Self {
            memory_buffer: memory_buffer,
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
}
