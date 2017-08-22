pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub const SPRITES: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0 /* 0 */, 0x20, 0x60, 0x20, 0x20,
                               0x70 /* 1 */, 0xF0, 0x10, 0xF0, 0x80, 0xF0 /* 2 */, 0xF0,
                               0x10, 0xF0, 0x10, 0xF0 /* 3 */, 0x90, 0x90, 0xF0, 0x10,
                               0x10 /* 4 */, 0xF0, 0x80, 0xF0, 0x10, 0xF0 /* 5 */, 0xF0,
                               0x80, 0xF0, 0x90, 0xF0 /* 6 */, 0xF0, 0x10, 0x20, 0x40,
                               0x40 /* 7 */, 0xF0, 0x90, 0xF0, 0x90, 0xF0 /* 8 */, 0xF0,
                               0x90, 0xF0, 0x10, 0xF0 /* 9 */, 0xF0, 0x90, 0xF0, 0x90,
                               0x90 /* a */, 0xE0, 0x90, 0xE0, 0x90, 0xE0 /* b */, 0xF0,
                               0x80, 0x80, 0x80, 0xF0 /* c */, 0xE0, 0x90, 0x90, 0x90,
                               0xE0 /* d */, 0xF0, 0x80, 0xF0, 0x80, 0xF0 /* e */, 0xF0,
                               0x80, 0xF0, 0x80, 0x80];// f

pub type Screen = [[bool; WIDTH]; HEIGHT];

pub struct Display {
    screen: Screen,
}

impl Display {
    pub fn new() -> Display {
        Display {
            screen: [[false; WIDTH]; HEIGHT],
        }
    }

    pub fn get_screen(&mut self) -> &Screen {
        &self.screen
    }

    // TODO: Please find a better way to do this...it hurts.
    pub fn draw_sprite(&mut self, sprite: &[u8], x: usize, y: usize) {
        for (i, row) in sprite.iter().enumerate() {
            println!("Drawing sprite row at ({}, {}) => {:b}", y, x, row);
            // Ex. 11110000 -> 0 0 0 0 1 1 1 1
            // Because we get them in the reverse order of the indexing
            // we need to use "7-j" for the column index.
            for j in 0..8 {
                let pixel_val = (row >> j) & 0b1;
                self.screen[ y + i ][ x + (7-j)] = if pixel_val == 0 {
                    false
                } else {
                    true
                };
            }
        }
    }
}
