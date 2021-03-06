extern crate sdl2;

use sdl2::video::Window;
use std::io::{Error, Read};

use crate::cpu::*;
use crate::display::*;
use crate::keypad::*;
use crate::memory::*;
use crate::speaker::*;

pub struct Emulator {
    pub cpu: Cpu,
    pub memory: Memory,
    pub keypad: Keypad,
    pub display: Display,
    pub speaker: Speaker,

    iteration: u64,
}

impl Emulator {
    // Constructors
    pub fn new(
        clock_rate: f32,
        ignore_unknown_instructions: bool,
        program_address: usize,
        display_scale: u8,
    ) -> Emulator {
        Emulator {
            cpu: Cpu::new(clock_rate, ignore_unknown_instructions, program_address),
            memory: Memory::new(),
            keypad: Keypad::new(),
            display: Display::new(display_scale),
            speaker: Speaker::new(),

            iteration: 1,
        }
    }

    // Methods
    pub fn load_rom(&mut self, reader: &mut dyn Read) -> Result<usize, Error> {
        self.iteration = 1;
        self.keypad.reset();
        self.display.clear();
        self.speaker.clear_queue();

        self.cpu.load_rom(&mut self.memory, reader)
    }

    pub fn step(
        &mut self,
        delta_time: f32,
        mut renderer: &mut sdl2::render::Canvas<Window>,
        sound: bool,
        debug_cpu: bool,
        debug_memory: bool,
    ) {
        // Debugging
        if debug_cpu || debug_memory {
            println!("\nIteration #{}", self.iteration);
        }

        // CPU
        self.cpu.step(
            delta_time,
            &mut self.memory,
            &mut self.keypad,
            &mut self.display,
            &mut self.speaker,
            debug_cpu,
            debug_memory,
        );

        // Other devices
        if self.display.needs_redraw() || self.iteration == 1 {
            self.display.draw(&mut renderer);
        }

        if sound {
            self.speaker.flush_queue();
        }

        self.iteration += 1;
    }
}
