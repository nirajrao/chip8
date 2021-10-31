use sdl2::keyboard::Keycode;
use std::collections::HashMap;
use crate::constants::NUM_KEYS;

/// Keypad:
/// 1 | 2 | 3 | 4
/// --------------
/// Q | W | E | R
/// --------------
/// A | S | D | F
/// --------------
/// Z | X | C | V
fn initialize_keypad() -> HashMap<Keycode, u8> {
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

/// Returns an array of pressed keys that are part of the CHIP-8 Keypad.
pub fn process_key_presses(pressed_keys: Vec<Keycode>) -> [u8; NUM_KEYS] {
    let keypad = initialize_keypad();

    let mut keys: [u8; NUM_KEYS] = [0; NUM_KEYS];

    for pressed_key in pressed_keys{
        if keypad.contains_key(&pressed_key) {
            let index = keypad[&pressed_key];
            keys[index as usize] = 1;
        }

    }

    keys
}

