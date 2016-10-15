#[macro_use]
extern crate clap;
extern crate sdl2;

use clap::App;

use sdl2::event::Event;

use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;


mod emulator;
mod memory;
mod cpu;
mod keypad;
mod display;

use emulator::Emulator;

fn main() {
    // Load configuration
    // TODO: Implement memory_size, display_width and display_height parameters
    let yaml = load_yaml!("cli.yml");
    let parameters = App::from_yaml(yaml).get_matches();

    let rom = parameters.value_of("rom").unwrap();
    let clock_rate = value_t!(parameters, "clock_rate", f32).unwrap();
    let ignore_unknown_instructions = parameters.is_present("ignore_unknown_instructions");
    let program_address = value_t!(parameters, "program_address", usize).unwrap();
    let display_scale = value_t!(parameters, "display_scale", u8).unwrap();
    let debug_cpu = parameters.is_present("debug_cpu");
    let debug_memory = parameters.is_present("debug_memory");

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
    
	// Initialize SDL2
    let sdl2_context = sdl2::init().unwrap();
    let mut sdl2_events = sdl2_context.event_pump().unwrap();

    let sdl2_video = sdl2_context.video().unwrap();
    let window = emulator.display.create_window(& sdl2_video);
    let mut renderer =  window.renderer().build().unwrap();

    let sdl2_timing = sdl2_context.timer().unwrap();
  
    // Game loop
    let mut last_step_time = get_time(&sdl2_timing);

    'running: loop {
        let processing_start = get_time(&sdl2_timing);

        // Events
        for event in sdl2_events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => emulator.keypad.key_down(keycode),
                Event::KeyUp { keycode: Some(keycode), .. }   => emulator.keypad.key_up(keycode),
                _ => ()
            }
        }

        // Emulation
        let delta_time = (get_time(&sdl2_timing) - last_step_time) * 1000 / sdl2_timing.performance_frequency();
        emulator.step(delta_time as f32, &mut renderer, debug_cpu, debug_memory);

        let frame_wait_duration = 1.0 / emulator.get_cpu_clock_rate() * 1000.0;
        let processing_time = (get_time(&sdl2_timing) - processing_start) * 1000 / sdl2_timing.performance_frequency();
        let sleep_time = std::cmp::max(frame_wait_duration as u32 - processing_time as u32, 0);

        last_step_time = get_time(&sdl2_timing);
        thread::sleep(Duration::new(0, sleep_time * 1000000));
    }
}

fn get_time(sdl2_timing: &sdl2::TimerSubsystem) -> u64 {
    sdl2_timing.performance_counter()
}
