use sdl2::keyboard::Keycode;
use std::collections::HashMap;

pub struct Keypad {}

impl Keypad {
    pub fn new() -> HashMap<Keycode, u8> {
        let mut keypad = HashMap::new();

        keypad.insert(Keycode::Num1, 0x1);
        keypad.insert(Keycode::Num2, 0x2);
        keypad.insert(Keycode::Num3, 0x3);
        keypad.insert(Keycode::Num4, 0xC);
        keypad.insert(Keycode::Q, 0x4);
        keypad.insert(Keycode::W, 0x5);
        keypad.insert(Keycode::E, 0x6);
        keypad.insert(Keycode::R, 0xD);
        keypad.insert(Keycode::A, 0x7);
        keypad.insert(Keycode::S, 0x8);
        keypad.insert(Keycode::D, 0x9);
        keypad.insert(Keycode::F, 0xE);
        keypad.insert(Keycode::Z, 0xA);
        keypad.insert(Keycode::X, 0x0);
        keypad.insert(Keycode::C, 0xB);
        keypad.insert(Keycode::V, 0xF);

        keypad
    }
}






