extern crate piston_window;
extern crate image;
extern crate rand;

mod cpu;
mod display;
mod instruction;

use piston_window::*;
use cpu::Cpu;
use std::fs::File;
use std::io::Read;
use std::env;

const ENLARGEMENT_FACTOR: usize = 1;
const WINDOW_WIDTH: u32 = 64;
const WINDOW_HEIGHT: u32 = 32;

fn main() {
    let args: Vec<String> = env::args().collect();
    let debug = args.len() > 1 && args[1] == "debug";

    let width = WINDOW_WIDTH;
    let height = WINDOW_HEIGHT;

    let mut window = create_window(width, height);

    let mut file = File::open("/home/matthew/development/chip8-roms/maze.rom").expect("Unable to open the ROM file.");
    let mut game_data = Vec::new();
    file.read_to_end(&mut game_data).expect("Unable to read the ROM file.");

    let mut cpu = Cpu::new(game_data);

    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            draw_screen(&e, &cpu.get_screen(), &mut window);
        }
        if let Some(button) = e.press_args() {
            if button == Button::Keyboard(Key::Space) && debug {
                cpu.cycle();
            }
        };
        
        // If debugging is enabled, only cycle on space bar presses
        if !debug {
            cpu.cycle();
        }
        
    }
}

fn create_window(width: u32, height: u32) -> PistonWindow {
    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow = WindowSettings::new("chip8", (width, height))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    window.set_max_fps(60);

    window
}

fn draw_screen(event: &Event, screen: &display::Screen, window: &mut PistonWindow) {
    window.draw_2d(event, |context, graphics| {
        piston_window::clear(color::BLACK, graphics);

        for (i, row) in screen.iter().enumerate() {
            for (j, val) in row.iter().enumerate() {
                if *val {
                    let dimensions = [
                        (j * ENLARGEMENT_FACTOR) as f64,
                        (i * ENLARGEMENT_FACTOR) as f64,
                        ENLARGEMENT_FACTOR as f64,
                        ENLARGEMENT_FACTOR as f64,
                    ];
                    Rectangle::new(color::WHITE).draw(
                        dimensions,
                        &context.draw_state,
                        context.transform,
                        graphics,
                    );
                }
            }
        }
    });
}
