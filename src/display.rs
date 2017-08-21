pub const WINDOW_WIDTH: usize = 300;
pub const WINDOW_HEIGHT: usize = 300;

pub type Screen = [[bool; WINDOW_WIDTH]; WINDOW_HEIGHT];

pub struct Display {
    screen: Screen,
}

impl Display {
    pub fn new() -> Display {
        Display {
            screen: [[false; WINDOW_WIDTH]; WINDOW_HEIGHT],
        }
    }

    pub fn get_screen(&mut self) -> Screen {
        self.screen
    }
}
