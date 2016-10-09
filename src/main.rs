// RUST implementation of a Chip-8 interpreter

#[macro_use]
extern crate clap;
extern crate sdl2;

use clap::App;

use sdl2::event::Event;

use std::fs::File;
use std::path::Path;
use std::thread;


mod emulator;
mod memory;
mod cpu;
mod keypad;
mod display;

use emulator::Emulator;

fn main() {
    // Load configuration
    // TODO: Fix flag parameters
    // TODO: Implement memory_size, display_width and display_height parameters
    let yaml = load_yaml!("cli.yml");
    let parameters = App::from_yaml(yaml).get_matches();

    let rom = parameters.value_of("rom").unwrap();
    let clock_rate = value_t!(parameters, "clock_rate", f32).unwrap();
    let ignore_unknown_instructions = parameters.value_of("ignore_unknown_instructions").is_some();
    let program_address = value_t!(parameters, "program_address", usize).unwrap();
    let display_scale = value_t!(parameters, "display_scale", u8).unwrap();
    let debug = parameters.value_of("debug").is_some();

    if clock_rate <= 0.0 {
        panic!("parameter \"clock_rate\" must be > 0");
    }
    if display_scale <= 0 {
        panic!("parameter \"display_scale\" must be > 0");
    }

	// Initialize emulator
	let mut emulator = Emulator::new(
        clock_rate, ignore_unknown_instructions, program_address,
        display_scale);

	let mut rom_file = File::open(&Path::new(rom)).unwrap();
	emulator.load_rom(&mut rom_file).unwrap();
    
	// Game loop
    // TODO: Find solution for multiple SDL2 instances needed but not possible
    //let sdl_context = sdl2::init().unwrap();
    //let mut events = sdl_context.event_pump().unwrap();

    let mut iteration: u64 = 0;
    //let mut last_frame = sdl2::
    
    'running: loop {
        // Events
        /*for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => emulator.keypad.key_down(keycode),
                Event::KeyUp { keycode: Some(keycode), .. }   => emulator.keypad.key_up(keycode),
                _ => ()
            }
        }*/

        // Emulation
        emulator.step(1.0 / 60.0 / 10.0, debug);
        thread::sleep_ms(16);

        iteration += 1;
    }
}
