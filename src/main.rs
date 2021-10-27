mod chip;
mod opcode;
use sdl2;
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

    let mut chip8 = chip::Chip8::new("roms/bc_test.ch8");

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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }
}
