mod chip;

fn main() {
    let mut chip8 = chip::Chip8::new("pong.ch8");
    println!("{}", chip8.fetch_opcode());
}
