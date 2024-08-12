use crate::vm;
use sdl2::keyboard::Keycode;

pub fn keycode_to_vm_input(keycode: Keycode) -> u8 {
    match keycode {
        Keycode::Up => vm::INPUT_UP,
        Keycode::Down => vm::INPUT_DOWN,
        Keycode::Left => vm::INPUT_LEFT,
        Keycode::Right => vm::INPUT_RIGHT,
        Keycode::Return => vm::INPUT_START,
        Keycode::Space => vm::INPUT_SELECT,
        Keycode::A => vm::INPUT_A,
        Keycode::B => vm::INPUT_B,
        _ => vm::INPUT_NONE,
    }
}
