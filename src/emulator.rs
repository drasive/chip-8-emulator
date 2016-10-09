extern crate sdl2;

use std::io::{Read, Error};

use cpu::Cpu;
use memory::Memory;
use keypad::Keypad;
use display::Display;


pub struct Emulator {
    pub cpu: Cpu,
    pub memory: Memory,    
    pub keypad: Keypad,
    pub display: Display,
}


impl Emulator {

    // Constructors
    pub fn new(
        clock_rate: f32, ignore_unknown_instructions: bool, program_address: usize,
        display_scale: u8) -> Emulator {

        Emulator {
            cpu: Cpu::new(clock_rate, ignore_unknown_instructions, program_address),
            memory: Memory::new(),
            keypad: Keypad::new(),
            display: Display::new(display_scale)
        }
    }


    // Methods
    pub fn load_rom(&mut self, reader: &mut Read) -> Result<usize, Error> {
        self.cpu.load_rom(&mut self.memory, reader)
    }

    pub fn step(&mut self, delta_time: f32, mut renderer: &mut sdl2::render::Renderer, debug_cpu: bool, debug_memory: bool) {
        self.cpu.step(&mut self.memory, &mut self.keypad, &mut self.display, delta_time, debug_cpu, debug_memory);
        self.display.draw(&mut renderer);
    }

}
