mod chip;
mod opcode;
mod keypad;
use sdl2::keyboard::Keycode;
use keypad::Keypad;
use std::time::Duration;

const SCREEN_WIDTH: usize = 640;
const SCREEN_HEIGHT: usize = 320;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem.window("Game", 900, 700).resizable().build().unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl.event_pump().unwrap();

    let mut chip8 = chip::Chip8::new("roms/random_number_test.ch8");

    let keypad = Keypad::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let keys: Vec<Keycode> = event_pump.keyboard_state().pressed_scancodes().filter_map(Keycode::from_scancode).collect();

        for key in keys {
            if keypad.contains_key(&key) {
                let index = keypad[&key];
                chip8.keys[index as usize] = 1;
            }
        }

        chip8.emulate_cycle();
        for i in 0..64{
            for j in 0..32{
                if chip8.graphics[i][j] == 1 {
                    let rect: sdl2::rect::Rect = sdl2::rect::Rect::new((i * 10) as i32, (j * 10) as i32, 10, 10);
                    canvas.draw_rect(rect).unwrap();
                }
            }
        }

        canvas.present();
        chip8.keys = [0; 16];
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
}
