extern crate piston_window;
extern crate image;
extern crate rand;

mod cpu;
mod display;
mod instruction;
mod keyboard;

use piston_window::*;
use cpu::Cpu;
use keyboard::{ Keyboard, KeyMapping };
use std::fs::File;
use std::io::Read;
use std::env;

const ENLARGEMENT_FACTOR: u32 = 8;
const WINDOW_WIDTH: u32 = 64;
const WINDOW_HEIGHT: u32 = 32;

fn main() {
    let args: Vec<String> = env::args().collect();
    let debug = args.len() > 1 && args[1] == "debug";
    let step = args.len() > 2 && args[2] == "step";

    let width = WINDOW_WIDTH * ENLARGEMENT_FACTOR;
    let height = WINDOW_HEIGHT * ENLARGEMENT_FACTOR;

    let mut window = create_window(width, height);

    let mut file = File::open("/home/matt/Development/chip8/roms/15puzzle.rom")
        .expect("Unable to open the ROM file.");

    let mut game_data = Vec::new();
    file.read_to_end(&mut game_data).expect(
        "Unable to read the ROM file.",
    );

    let mut cpu = Cpu::new(game_data, debug);
    let keyboard = Keyboard::new(KeyMapping::Improved);

    while let Some(e) = window.next() {

        let mut step_forward = false;

        if let Some(_) = e.render_args() {
            draw_screen(&e, &cpu.get_screen(), &mut window);
        }

        if let Some(button) = e.press_args() {
            if button == Button::Keyboard(Key::Space) && step {
                step_forward = true;
            }

            if let Some(key_val) = keyboard.map_key(button) {
                cpu.set_key(key_val, true);
            }
        }

        if let Some(button) = e.release_args() {
            if let Some(key_val) = keyboard.map_key(button) {
                cpu.set_key(key_val, false);
            }
        }
        
        // If debugging is enabled, only cycle on space bar presses
        if !step || step_forward {
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
                        (j * ENLARGEMENT_FACTOR as usize) as f64,
                        (i * ENLARGEMENT_FACTOR as usize) as f64,
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


// TODO:
// 1) Fix display issues, it is currently not working at all
//     a) This might (and probably should) involve rewriting the display logic
// 2) Add command line argument for ROM file
// 3) Fix command line arguments, potentially add a create for arg parsing (docopt?)
// 4) Keyboard scheme?
// 5) The display should really be separate from the cpu....not sure how to do this though