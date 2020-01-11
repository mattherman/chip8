extern crate piston_window;
extern crate image;
extern crate rand;

#[macro_use]
extern crate clap;

mod cpu;
mod display;
mod instruction;
mod keyboard;

use clap::App;
use piston_window::*;
use cpu::Cpu;
use keyboard::{ Keyboard, KeyMapping };
use std::fs::File;
use std::io::Read;
use std::process;
use std::time::Instant;

const ENLARGEMENT_FACTOR: u32 = 8;
const WINDOW_WIDTH: u32 = 64;
const WINDOW_HEIGHT: u32 = 32;

const CLOCK_SPEED_HZ_HALF: u32 = 180;
const CLOCK_SPEED_HZ_DEFAULT: u32 = 360;
const CLOCK_SPEED_HZ_DOUBLE: u32 = 720;

enum EmulatorSpeed {
    Half,
    Normal,
    Double
}

struct Arguments {
    rom: String,
    step: bool,
    debug: bool,
    speed: EmulatorSpeed
}

fn main() {
    let arguments = match parse_args() {
        Ok(args) => args,
        Err(_) => {
            println!("Usage: chip8 [rom] [--debug (optional)] [--step (optional)]");
            process::exit(0);
        }
    };

    let width = WINDOW_WIDTH * ENLARGEMENT_FACTOR;
    let height = WINDOW_HEIGHT * ENLARGEMENT_FACTOR;

    let mut window = create_window(width, height);

    let mut file = File::open(arguments.rom)
        .expect("Unable to open the ROM file.");

    let mut game_data = Vec::new();
    file.read_to_end(&mut game_data)
        .expect("Unable to read the ROM file.",);

    let clock_speed = match arguments.speed {
        EmulatorSpeed::Half => CLOCK_SPEED_HZ_HALF,
        EmulatorSpeed::Normal => CLOCK_SPEED_HZ_DEFAULT,
        EmulatorSpeed::Double => CLOCK_SPEED_HZ_DOUBLE
    };

    let mut cpu = Cpu::new(game_data, clock_speed, arguments.debug);
    let keyboard = Keyboard::new(KeyMapping::Improved);

    let cycle_time_millis: u128 = (1000 / clock_speed).into();

    let mut clock = Instant::now();
    while let Some(e) = window.next() {

        let mut step_forward = false;

        if let Some(_) = e.render_args() {
            if cpu.draw_flag {
                draw_screen(&e, &cpu.get_screen(), &mut window);
                cpu.draw_flag = false
            }
        }

        if let Some(button) = e.press_args() {
            if button == Button::Keyboard(Key::Space) && arguments.step {
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
        if arguments.step {
            if step_forward {
                cpu.cycle();
            }
        } else {
            let elapsed = clock.elapsed().as_millis();
            if elapsed >= cycle_time_millis {
                cpu.cycle();
                clock = Instant::now();
            }
        }
    }
}

fn parse_args() -> Result<Arguments, String> {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let rom = String::from(matches.value_of("ROM").unwrap());
    let debug = matches.is_present("debug");
    let step = matches.is_present("step");
    let speed_arg = matches.value_of("speed").unwrap_or("1");
    let speed = match speed_arg {
        "0.5" => EmulatorSpeed::Half,
        "2" => EmulatorSpeed::Double,
        _ => EmulatorSpeed::Normal
    };

    let args = Arguments { rom, step, debug, speed };
    return Ok(args)
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
// 4) Keyboard scheme?
// 5) The display should really be separate from the cpu....not sure how to do this though