mod chip;
mod opcode;
mod keypad;
mod display;
mod constants;

use sdl2::keyboard::Keycode;
use std::path::Path;
use display::Display;
use keypad::process_key_presses;
use sdl2::event::Event;
use std::time::Duration;

const ROM_PATH: &str = "./roms";

pub fn main() -> Result<(), String> {
    let filename = std::env::args().nth(1).expect("No ROM filename was passed in");
    let path = Path::new(ROM_PATH).join(filename);

    let mut chip8 = chip::Chip8::new(path);

    let mut display = Display::new();

    let mut event_pump = display.initialize_event_pump();

    display.present_canvas();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let pressed_keys = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let keys = process_key_presses(pressed_keys);

        chip8.emulate_cycle(keys);

        display.update_canvas(&chip8);

        display.present_canvas();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }

    Ok(())
}
