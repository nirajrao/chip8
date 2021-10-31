use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;
use sdl2::rect::Rect;
use sdl2::EventPump;

use crate::chip::Chip8;
use crate::constants::{SCREEN_WIDTH, SCREEN_HEIGHT, PIXEL_RATIO};

pub struct Display {
    sdl: Sdl,
    canvas: Canvas<Window>,
}

impl Display {
    pub fn new() -> Self {
        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem.window("CHIP-8 Emulator", PIXEL_RATIO * SCREEN_WIDTH, PIXEL_RATIO * SCREEN_HEIGHT).resizable().opengl().build().unwrap();

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        Self {
            sdl,
            canvas,
        }
    }

    pub fn update_canvas(&mut self, chip8: &Chip8) {
        for i in 0..SCREEN_WIDTH{
            for j in 0..SCREEN_HEIGHT{
                if chip8.graphics[i as usize][j as usize] == 1 {
                    let rect: Rect = Rect::new((i * PIXEL_RATIO) as i32, (j * PIXEL_RATIO) as i32, PIXEL_RATIO, PIXEL_RATIO);
                    self.canvas.draw_rect(rect).unwrap();
                }
            }
        }
    }

    pub fn initialize_event_pump(&self) -> EventPump {
        self.sdl.event_pump().unwrap()
    }

    pub fn present_canvas(&mut self) {
        self.canvas.present();
    }

}

