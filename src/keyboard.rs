use piston_window::{Button, Key};

pub struct Keyboard {
    mapping: KeyMapping,
}

#[allow(dead_code)]
pub enum KeyMapping {
    Default,
    Improved,
}

impl Keyboard {
    pub fn new(map: KeyMapping) -> Keyboard {
        Keyboard { mapping: map }
    }

    pub fn map_key(&self, button: Button) -> Option<u8> {
        match self.mapping {
            KeyMapping::Default => self.default_keymapping(button),
            KeyMapping::Improved => self.improved_keymapping(button),
        }
    }

    fn default_keymapping(&self, button: Button) -> Option<u8> {
        match button {
            Button::Keyboard(Key::D0) => Some(0x0),
            Button::Keyboard(Key::D1) => Some(0x1),
            Button::Keyboard(Key::D2) => Some(0x2),
            Button::Keyboard(Key::D3) => Some(0x3),
            Button::Keyboard(Key::D4) => Some(0x4),
            Button::Keyboard(Key::D5) => Some(0x5),
            Button::Keyboard(Key::D6) => Some(0x6),
            Button::Keyboard(Key::D7) => Some(0x7),
            Button::Keyboard(Key::D8) => Some(0x8),
            Button::Keyboard(Key::D9) => Some(0x9),
            Button::Keyboard(Key::A) => Some(0xA),
            Button::Keyboard(Key::B) => Some(0xB),
            Button::Keyboard(Key::C) => Some(0xC),
            Button::Keyboard(Key::D) => Some(0xD),
            Button::Keyboard(Key::E) => Some(0xE),
            Button::Keyboard(Key::F) => Some(0xF),
            _ => None,
        }
    }

    fn improved_keymapping(&self, button: Button) -> Option<u8> {
        match button {
            Button::Keyboard(Key::D1) => Some(0x1),
            Button::Keyboard(Key::D2) => Some(0x2),
            Button::Keyboard(Key::D3) => Some(0x3),
            Button::Keyboard(Key::D4) => Some(0xC),
            Button::Keyboard(Key::Q) => Some(0x4),
            Button::Keyboard(Key::W) => Some(0x5),
            Button::Keyboard(Key::E) => Some(0x6),
            Button::Keyboard(Key::R) => Some(0xD),
            Button::Keyboard(Key::A) => Some(0x7),
            Button::Keyboard(Key::S) => Some(0x8),
            Button::Keyboard(Key::D) => Some(0x9),
            Button::Keyboard(Key::F) => Some(0xE),
            Button::Keyboard(Key::Z) => Some(0xA),
            Button::Keyboard(Key::X) => Some(0x0),
            Button::Keyboard(Key::C) => Some(0xB),
            Button::Keyboard(Key::V) => Some(0xF),
            _ => None,
        }
    }
}
