use piston_window::{ Button, Key };

pub fn map_key(button: Button) -> Option<u8> {
    match button {
        Button::Keyboard(Key::NumPad0) => Some(0x0),
        Button::Keyboard(Key::NumPad1) => Some(0x1),
        Button::Keyboard(Key::NumPad2) => Some(0x2),
        Button::Keyboard(Key::NumPad3) => Some(0x3),
        Button::Keyboard(Key::NumPad4) => Some(0x4),
        Button::Keyboard(Key::NumPad5) => Some(0x5),
        Button::Keyboard(Key::NumPad6) => Some(0x6),
        Button::Keyboard(Key::NumPad7) => Some(0x7),
        Button::Keyboard(Key::NumPad8) => Some(0x8),
        Button::Keyboard(Key::NumPad9) => Some(0x9),
        Button::Keyboard(Key::A) => Some(0xA),
        Button::Keyboard(Key::B) => Some(0xB),
        Button::Keyboard(Key::C) => Some(0xC),
        Button::Keyboard(Key::D) => Some(0xD),
        Button::Keyboard(Key::E) => Some(0xE),
        Button::Keyboard(Key::F) => Some(0xF),
        _ => None,
    }
}