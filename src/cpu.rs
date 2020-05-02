extern crate rand;
extern crate sdl2;

use std::io::{BufWriter, Error, Read, Write};

use crate::display::DisplayTrait;
use crate::keypad::KeypadTrait;
use crate::memory::MemoryTrait;
use crate::speaker::SpeakerTrait;

// Font data
const FONT_WIDTH: usize = 5;
const FONT_BYTES: usize = FONT_WIDTH * 16;

const FONT: [u8; FONT_BYTES] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub trait CpuTrait {
    fn load_rom(
        &mut self,
        memory: &mut dyn MemoryTrait,
        rom_reader: &mut dyn Read,
    ) -> Result<usize, Error>;
    fn step(
        &mut self,
        delta_time: f32,
        memory: &mut dyn MemoryTrait,
        keypad: &mut dyn KeypadTrait,
        display: &mut dyn DisplayTrait,
        speaker: &mut dyn SpeakerTrait,
        debug_cpu: bool,
        debug_memory: bool,
    );
    fn get_clock_rate(&self) -> f32;
    fn print_debug_info(&self);
}

pub struct Cpu {
    // Program
    opcode: u16, // current opcode (two 8-bit values)
    pc: usize,   // 16-bit program counter

    // Registers
    v: [u8; 16], // 16 8-bit general purpose registers
    i: u16,      // 16-bit register for storing memory adresses

    delay_timer: u8,    // 8-bit delay timer (decremented at 60 Hz)
    delay_timer_f: f32, // float representation of delay_timer
    sound_timer: u8,    // 8-bit sound timer (decremented at 60 Hz)
    sound_timer_f: f32, // float representation of sound_timer

    // Stack
    stack: [usize; 16], // 16 values to store return values of subroutines
    sp: u8,             // 8-bit register that points to the topmost level of the stack

    // Configuration
    clock_rate: f32,
    ignore_unknown_instructions: bool,
    program_address: usize,
}

impl CpuTrait for Cpu {
    fn load_rom(
        &mut self,
        memory: &mut dyn MemoryTrait,
        rom_reader: &mut dyn Read,
    ) -> Result<usize, Error> {
        // Clear memory
        memory.clear();

        // Copy font to memory at 0x000
        if FONT_BYTES > memory.get_size() {
            panic!("Font size ({font_bytes} bytes) is larger than available memory ({memory_size} bytes)",
                font_bytes = FONT_BYTES, memory_size = memory.get_size());
        }

        {
            println!(
                "Copying font ({font_bytes} bytes) to memory at 0x000",
                font_bytes = FONT_BYTES
            );
            let mut memory_stream = BufWriter::new(&mut memory.read_all()[0..FONT_BYTES]);
            memory_stream.write_all(FONT.as_ref()).unwrap();
        }

        // Read ROM
        println!("Reading ROM");
        let mut rom = Vec::new();
        r#try!(rom_reader.read_to_end(&mut rom));

        // Copy ROM into memory
        if rom.len() < 2 {
            panic!("ROM does not contain any instructions");
        } else if rom.len() > memory.get_size() - self.program_address {
            panic!("ROM size ({rom_size} bytes) is larger than available program memory ({available_memory}) bytes)",
                   rom_size = rom.len(), available_memory = memory.get_size() - self.program_address);
        }
        {
            println!(
                "Copying ROM ({rom_size} bytes) to memory at 0x{program_start:X}",
                rom_size = rom.len(),
                program_start = self.program_address
            );
            let mut memory_stream = BufWriter::new(
                &mut memory.read_all()[self.program_address..(self.program_address + rom.len())],
            );
            r#try!(memory_stream.write_all(rom.as_ref()));
        }

        self.pc = self.program_address;
        return Ok(rom.len());
    }

    fn step(
        &mut self,
        delta_time: f32,
        memory: &mut dyn MemoryTrait,
        keypad: &mut dyn KeypadTrait,
        display: &mut dyn DisplayTrait,
        speaker: &mut dyn SpeakerTrait,
        debug_cpu: bool,
        debug_memory: bool,
    ) {
        // Fetch opcode
        self.opcode = (memory.read(self.pc) as u16) << 8 | (memory.read(self.pc + 1) as u16);

        // Debugging
        if debug_cpu {
            self.print_debug_info();
        }
        if debug_memory {
            memory.print_debug_info();
        }

        // Execute opcode
        self.execute_instruction(memory, keypad, display);

        // Periodic tasks
        self.update_delay_timer(delta_time);
        self.update_sound_timer(delta_time, speaker);
    }

    fn get_clock_rate(&self) -> f32 {
        self.clock_rate
    }

    fn print_debug_info(&self) {
        let opname = Cpu::get_opname(&self.opcode);
        println!(
            "Op: 0x{:X} {}, PC: {}, I: 0x{:X}, DT: {}, ST: {}",
            self.opcode, opname, self.pc, self.i, self.delay_timer, self.sound_timer
        );

        println!("Registers: {:?}", self.v);
        println!("Stack: {:?}", self.stack);
    }
}

impl Cpu {
    pub fn new(clock_rate: f32, ignore_unknown_instructions: bool, program_address: usize) -> Cpu {
        println!(
            "Initializing processor with {clock_rate} Hz",
            clock_rate = clock_rate
        );

        Cpu {
            // Program
            opcode: 0x00,
            pc: 0,

            // Registers
            v: [0; 16],
            i: 0,

            delay_timer: 0,
            delay_timer_f: 0.0,
            sound_timer: 0,
            sound_timer_f: 0.0,

            // Stack
            stack: [0; 16],
            sp: 0,

            // Configuration
            clock_rate: clock_rate,
            ignore_unknown_instructions: ignore_unknown_instructions,
            program_address: program_address,
        }
    }

    fn execute_instruction(
        &mut self,
        memory: &mut dyn MemoryTrait,
        keypad: &mut dyn KeypadTrait,
        display: &mut dyn DisplayTrait,
    ) {
        let byte_1 = (self.opcode & 0xF000) >> 0xC;
        let byte_2 = ((self.opcode & 0x0F00) >> 0x8) as usize;
        let byte_3 = ((self.opcode & 0x00F0) >> 0x4) as usize;
        let byte_4 = self.opcode & 0x000F;

        match (byte_1, byte_2, byte_3, byte_4) {
            (0x0, 0x0, 0xE, 0x0) => {
                // 00E0 - CLS; Clear the display.

                display.clear();

                self.pc += 2;
            }
            (0x0, 0x0, 0xE, 0xE) => {
                // 00EE - RET; Return from a subroutine.
                // The interpreter sets the program counter to the address at the top of the stack,
                // then subtracts 1 from the stack pointer.

                self.pc = self.stack[self.sp as usize] as usize;
                self.sp -= 1;

                self.pc += 2;
            }
            (0x1, _, _, _) => {
                // 1nnn - JP addr; Jump to location nnn.
                // The interpreter sets the program counter to nnn.

                self.pc = self.op_0nnn();
            }
            (0x2, _, _, _) => {
                // 2nnn - CALL addr; Call subroutine at nnn.
                // The interpreter increments the stack pointer, then puts the current PC on the top of the stack.
                // The PC is then set to nnn.

                self.sp += 1;
                self.stack[self.sp as usize] = self.pc as usize;
                self.pc = self.op_0nnn();
            }
            (0x3, x, _, _) => {
                // 3xkk - SE Vx, byte; Skip next instruction if Vx = kk.
                // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.

                if self.v[x] == self.op_00kk() {
                    self.pc += 2 * 2;
                } else {
                    self.pc += 2;
                }
            }
            (0x4, x, _, _) => {
                // 4xkk - SNE Vx, byte; Skip next instruction if Vx != kk.
                // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.

                if self.v[x] != self.op_00kk() {
                    self.pc += 2 * 2;
                } else {
                    self.pc += 2;
                }
            }
            (0x5, x, y, 0x0) => {
                // 5xy0 - SE Vx, Vy; Skip next instruction if Vx = Vy.
                // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.

                if self.v[x] == self.v[y] {
                    self.pc += 2 * 2;
                } else {
                    self.pc += 2;
                }
            }
            (0x6, x, _, _) => {
                // 6xkk - LD Vx, byte; Set Vx = kk.
                // The interpreter puts the value kk into register Vx.

                self.v[x] = self.op_00kk();

                self.pc += 2;
            }
            (0x7, x, _, _) => {
                // 7xkk - ADD Vx, byte; Set Vx = Vx + kk.
                // Adds the value kk to the value of register Vx, then stores the result in Vx.

                self.v[x] = self.v[x].wrapping_add(self.op_00kk());

                self.pc += 2;
            }
            (0x8, x, y, 0x0) => {
                // 8xy0 - LD Vx, Vy; Set Vx = Vy.
                // Stores the value of register Vy in register Vx.

                self.v[x] = self.v[y];

                self.pc += 2;
            }
            (0x8, x, y, 0x1) => {
                // 8xy1 - OR Vx, Vy; Set Vx = Vx OR Vy.
                // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.

                self.v[x] |= self.v[y];

                self.pc += 2;
            }
            (0x8, x, y, 0x2) => {
                // 8xy2 - AOR Vx, Vy; Set Vx = Vx AND Vy.
                // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.

                self.v[x] &= self.v[y];

                self.pc += 2;
            }
            (0x8, x, y, 0x3) => {
                // 8xy3 - XOR Vx, Vy; Set Vx = Vx XOR Vy.
                // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.

                self.v[x] ^= self.v[y];

                self.pc += 2;
            }
            (0x8, x, y, 0x4) => {
                // 8xy4 - ADD Vx, Vy; Set Vx = Vx + Vy, set VF = carry.
                // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,)
                // VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.

                let (result, flag) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = result as u8;
                self.v[0xF] = if flag { 1 } else { 0 };

                self.pc += 2;
            }
            (0x8, x, y, 0x5) => {
                // 8xy5 - SUB Vx, Vy; Set Vx = Vx - Vy, set VF = NOT borrow.
                // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.

                if self.v[x] > self.v[y] {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);

                self.pc += 2;
            }
            (0x8, x, _, 0x6) => {
                // 8xy6 - SHR Vx {, Vy}; Set Vx = Vx SHR 1.
                // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.

                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;

                self.pc += 2;
            }
            (0x8, x, y, 0x7) => {
                // 8xy7 - SUBN Vx, Vy; Set Vx = Vy - Vx, set VF = NOT borrow.
                // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
                if self.v[y] > self.v[x] {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);

                self.pc += 2;
            }
            (0x8, x, _, 0xE) => {
                // 8xyE - SHL Vx {, Vy}; Set Vx = Vx SHL 1.
                // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
                self.v[0xF] = (self.v[x] >> 7) & 0x1;
                self.v[x] <<= 1;

                self.pc += 2;
            }
            (0x9, x, y, 0x0) => {
                // 9xy0 - SNE Vx, Vy; Skip next instruction if Vx != Vy.
                // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.

                if self.v[x] != self.v[y] {
                    self.pc += 2 * 2;
                } else {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                // Annn - LD I, addr; Set I = nnn.
                // The value of register I is set to nnn.

                self.i = self.op_0nnn() as u16;

                self.pc += 2;
            }
            (0xB, _, _, _) => {
                // Bnnn - JP V0, addr; Jump to location nnn + V0.
                // The program counter is set to nnn plus the value of V0.

                self.pc = self.op_0nnn() + self.v[0x0] as usize;
            }
            (0xC, x, _, _) => {
                // Cxkk - RND Vx, byte; Set Vx = random byte AND kk.
                // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
                // The results are stored in Vx.

                // TODO: Use caching
                // Init once: let mut rng = rand::thread_rng();
                // Use: rng.gen::<u8>()

                self.v[x] = self.op_00kk() & rand::random::<u8>();

                self.pc += 2;
            }
            (0xD, x, y, n) => {
                // Dxyn - DRW Vx, Vy, nibble; Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                // The interpreter reads n bytes from memory, starting at the address stored in I.
                // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
                // Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1,
                // otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display,
                // it wraps around to the opposite side of the screen.

                let start = self.i as usize;
                let end = self.i as usize + n as usize;
                self.v[0xF] = display.draw_sprite(
                    self.v[x] as usize,
                    self.v[y] as usize,
                    &memory.read_all()[start..end],
                );

                self.pc += 2;
            }
            (0xE, x, 0x9, 0xE) => {
                // Ex9E - SKP Vx; Skip next instruction if key with the value of Vx is pressed.
                // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position,
                // PC is increased by 2.

                if keypad.get_key(x as u8) {
                    self.pc += 2 * 2;
                } else {
                    self.pc += 2;
                }
            }
            (0xE, x, 0xA, 0x1) => {
                // ExA1 - SKNP Vx; Skip next instruction if key with the value of Vx is not pressed.
                // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position,
                // PC is increased by 2.

                if !keypad.get_key(x as u8) {
                    self.pc += 2 * 2;
                } else {
                    self.pc += 2;
                }
            }
            (0xF, x, 0x0, 0x7) => {
                // Fx07 - LD Vx, DT; Set Vx = delay timer value.
                // The value of DT is placed into Vx.

                self.v[x] = self.delay_timer;

                self.pc += 2;
            }
            (0xF, x, 0x0, 0xA) => {
                // Fx0A - LD Vx, K; Wait for a key press, store the value of the key in Vx.
                // All execution stops until a key is pressed, then the value of that key is stored in Vx.

                for index in 0x0..0xF {
                    if keypad.get_key(index) {
                        self.v[x] = index;

                        self.pc += 2;
                        break;
                    }
                }
            }
            (0xF, x, 0x1, 0x5) => {
                // Fx15 - LD DT, Vx; Set delay timer = Vx.
                // DT is set equal to the value of Vx.

                self.delay_timer = self.v[x];
                self.delay_timer_f = self.delay_timer as f32;

                self.pc += 2;
            }
            (0xF, x, 0x1, 0x8) => {
                // Fx18 - LD ST, Vx; Set sound timer = Vx.
                // ST is set equal to the value of Vx.

                self.sound_timer = self.v[x];
                self.sound_timer_f = self.sound_timer as f32;

                self.pc += 2;
            }
            (0xF, x, 0x1, 0xE) => {
                // Fx1E - ADD I, Vx; Set I = I + Vx.
                // The values of I and Vx are added, and the results are stored in I.
                self.i += self.v[x] as u16;

                self.pc += 2;
            }
            (0xF, x, 0x2, 0x9) => {
                // Fx29 - LD F, Vx; Set I = location of sprite for digit Vx.
                // The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx.
                self.i = self.v[x] as u16 * FONT_WIDTH as u16;

                self.pc += 2;
            }
            (0xF, x, 0x3, 0x3) => {
                // Fx33 - LD B, Vx; Store BCD representation of Vx in memory locations I, I+1, and I+2.
                // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I,
                // the tens digit at location I+1, and the ones digit at location I+2.
                memory.write(self.i as usize, self.v[x] / 100);
                memory.write(self.i as usize + 1, (self.v[x] / 10) % 10);
                memory.write(self.i as usize + 2, (self.v[x] % 100) & 10);

                self.pc += 2;
            }
            (0xF, x, 0x5, 0x5) => {
                // Fx55 - LD [I], Vx; Store registers V0 through Vx in memory starting at location I.
                // The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.

                for index in 0..(x + 1) {
                    memory.write(self.i as usize + index, self.v[index] as u8);
                }
                self.i += x as u16 + 1;

                self.pc += 2;
            }
            (0xF, x, 0x6, 0x5) => {
                // Fx65 - LD Vx, [I]; Read registers V0 through Vx from memory starting at location I.
                // The interpreter reads values from memory starting at location I into registers V0 through Vx.

                for index in 0..(x + 1) {
                    self.v[index] = memory.read(self.i as usize + index) as u8;
                }
                self.i += x as u16 + 1;

                self.pc += 2;
            }
            _ => {
                // opcode "SYS" is intentionally not implemented
                if self.ignore_unknown_instructions {
                    println!("instruction not implemented. opcode: {opcode}, program counter: {program_counter}",
                             opcode = self.opcode, program_counter = self.pc);
                } else {
                    panic!("instruction not implemented. opcode: {opcode}, program counter: {program_counter}",
                           opcode = self.opcode, program_counter = self.pc);
                }
            }
        }
    }

    fn update_delay_timer(&mut self, delta_time: f32) {
        if self.delay_timer_f > 0.0 {
            self.delay_timer_f -= delta_time / 1000.0 / (1.0 / 60.0);
            if self.delay_timer_f < 0.0 {
                self.delay_timer_f = 0.0;
            }

            self.delay_timer = self.delay_timer_f.floor() as u8;
        }
    }

    fn update_sound_timer(&mut self, delta_time: f32, speaker: &mut dyn SpeakerTrait) {
        if self.sound_timer_f > 0.0 {
            self.sound_timer_f -= delta_time / 1000.0 / (1.0 / 60.0);
            if self.sound_timer_f < 0.0 {
                self.sound_timer_f = 0.0;
            }

            self.sound_timer = self.sound_timer_f.ceil() as u8;
            if self.sound_timer == 0 {
                speaker.queue_beep();
            }
        }
    }

    fn op_00kk(&mut self) -> u8 {
        (self.opcode & 0x00FF) as u8
    }

    fn op_0nnn(&mut self) -> usize {
        (self.opcode & 0x0FFF) as usize
    }

    fn get_opname(opcode: &u16) -> &str {
        let byte_1 = (opcode & 0xF000) >> 0xC;
        let byte_2 = ((opcode & 0x0F00) >> 0x8) as usize;
        let byte_3 = ((opcode & 0x00F0) >> 0x4) as usize;
        let byte_4 = opcode & 0x000F;

        match (byte_1, byte_2, byte_3, byte_4) {
            (0x0, 0x0, 0xE, 0x0) => "CLS",
            (0x0, 0x0, 0xE, 0xE) => "RET",
            (0x0, _, _, _) => "SYS",
            (0x1, _, _, _) => "JP (addr)",
            (0x2, _, _, _) => "CALL (addr)",
            (0x3, _, _, _) => "SE (Vx, byte)",
            (0x4, _, _, _) => "SNE (Vx, byte)",
            (0x5, _, _, 0x0) => "SE (Vx, Vy)",
            (0x6, _, _, _) => "LD (Vx, byte)",
            (0x7, _, _, _) => "ADD (Vx, byte)",
            (0x8, _, _, 0x0) => "LD (Vx, Vy)",
            (0x8, _, _, 0x1) => "OR (Vx, Vy)",
            (0x8, _, _, 0x2) => "AND (Vx, Vy)",
            (0x8, _, _, 0x3) => "XOR (Vx, Vy)",
            (0x8, _, _, 0x4) => "ADD (Vx, Vy)",
            (0x8, _, _, 0x5) => "SUB (Vx, Vy)",
            (0x8, _, _, 0x6) => "SHR (Vx, Vy)",
            (0x8, _, _, 0x7) => "SUBN (Vy, Vy)",
            (0x8, _, _, 0xE) => "SHL (Vx, Vy)",
            (0x9, _, _, 0x0) => "SNE (Vy, Vy)",
            (0xA, _, _, _) => "LD (I, addr)",
            (0xB, _, _, _) => "JP (V0, addr)",
            (0xC, _, _, _) => "RND (Vy, byte)",
            (0xD, _, _, _) => "DRW (Vx, Vy, nibble)",
            (0xE, _, 0x9, 0xE) => "SKP (Vx)",
            (0xE, _, 0xA, 0x1) => "SKNP (Vx)",
            (0xF, _, 0x0, 0x7) => "LD (Vx, DT)",
            (0xF, _, 0x0, 0xA) => "LD (Vx, K)",
            (0xF, _, 0x1, 0x5) => "LD (DT, Vx)",
            (0xF, _, 0x1, 0x8) => "LD (ST, Vx)",
            (0xF, _, 0x1, 0xE) => "ADD (I, Vx)",
            (0xF, _, 0x2, 0x9) => "LD (F, Vx)",
            (0xF, _, 0x3, 0x3) => "LD (B, Vx)",
            (0xF, _, 0x5, 0x5) => "LD (I, Vx)",
            (0xF, _, 0x6, 0x5) => "LD (Vx, I)",
            _ => "?",
        }
    }
}

// TODO: Split into separate file
#[cfg(test)]
mod tests {
    // Note: These tests cover more than just the CPU as they also integrate memory, display and keypad
    // 
    // Tests based on:
    // - https://github.com/starrhorne/chip8-rust/blob/master/src/processor_test.rs (accessed 2020-04-21)
    // - https://github.com/ismaelrh/Java-chip8-emulator/blob/master/src/test/java/chip8/ProcessingUnitTest.java (accessed 2020-04-21)
    
    use sdl2::keyboard::Keycode;
    
    use super::*;
    use crate::display::*;
    use crate::keypad::*;
    use crate::memory::*;

    const PROGRAM_START_ADDRESS: usize = 0x200;

    fn instantiate_cpu(memory: &mut dyn MemoryTrait) -> Cpu {
        instantiate_cpu_with_program(memory, vec![0x1200]) // Filler instruction
    }

    fn instantiate_cpu_with_program(memory: &mut dyn MemoryTrait, instructions: Vec<u16>) -> Cpu {
        let mut instructions_bytes: Vec<u8> = Vec::new();
        for instruction in instructions {
            instructions_bytes.extend(instruction.to_be_bytes().to_vec().into_iter());
        }

        let mut cpu = Cpu::new(600.0, false, PROGRAM_START_ADDRESS);
        cpu.load_rom(memory, &mut std::io::Cursor::new(instructions_bytes))
            .unwrap();
        cpu
    }

    fn instantiate_memory() -> Memory {
        Memory::new() // Not mocked dued to simplicity
    }

    fn instantiate_display() -> Display {
        Display::new(1) // Not mocked dued to simplicity
    }

    fn instantiate_keypad() -> Keypad {
        Keypad::new() // Not mocked dued to simplicity
    }

    fn execute_instruction(cpu: &mut Cpu, memory: &mut dyn MemoryTrait, opcode: u16) {
        let mut keypad = MockKeypadTrait::new();
        let mut display = MockDisplayTrait::new();

        cpu.opcode = opcode;
        cpu.execute_instruction(memory, &mut keypad, &mut display);
    }

    fn execute_instruction_with_display(
        cpu: &mut Cpu,
        memory: &mut dyn MemoryTrait,
        display: &mut dyn DisplayTrait,
        opcode: u16,
    ) {
        let mut keypad = MockKeypadTrait::new();

        cpu.opcode = opcode;
        cpu.execute_instruction(memory, &mut keypad, display);
    }

    fn execute_instruction_with_keypad(
        cpu: &mut Cpu,
        memory: &mut dyn MemoryTrait,
        keypad: &mut dyn KeypadTrait,
        opcode: u16,
    ) {
        let mut display = MockDisplayTrait::new();

        cpu.opcode = opcode;
        cpu.execute_instruction(memory, keypad, &mut display);
    }

    fn test_math(v1: u8, v2: u8, operation: u16, expected_result: u8, expected_vf: u8) {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0x0] = v1;
        cpu.v[0x1] = v2;
        cpu.v[0xf] = 0;
        execute_instruction(&mut cpu, &mut memory, 0x8010 + operation);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0x0], expected_result);
        assert_eq!(cpu.v[0xf], expected_vf);
    }

    #[test]
    fn test_initialize() {
        let mut memory = instantiate_memory();
        let cpu = instantiate_cpu(&mut memory);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.stack, [0; 16]);
        assert_eq!(cpu.i, 0);

        // First char in font: 0
        assert_eq!(memory.read_all()[0..5], [0xF0, 0x90, 0x90, 0x90, 0xF0]);

        // Last char in font: F
        assert_eq!(
            memory.read_all()[FONT.len() - 5..FONT.len()],
            [0xF0, 0x80, 0xF0, 0x80, 0x80]
        );
    }

    #[test]
    fn test_load_rom() {
        let mut memory = instantiate_memory();
        let _ = instantiate_cpu_with_program(&mut memory, vec![0x1234, 0x5678]);

        assert_eq!(memory.read(PROGRAM_START_ADDRESS + 0x0), 0x12);
        assert_eq!(memory.read(PROGRAM_START_ADDRESS + 0x1), 0x34);
        assert_eq!(memory.read(PROGRAM_START_ADDRESS + 0x2), 0x56);
        assert_eq!(memory.read(PROGRAM_START_ADDRESS + 0x3), 0x78);
        assert_eq!(memory.read(PROGRAM_START_ADDRESS + 0x4), 0x00);
    }

    #[test]
    fn test_call_and_ret() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.sp = 10;

        execute_instruction(&mut cpu, &mut memory, 0x2DAD);
        execute_instruction(&mut cpu, &mut memory, 0x00EE);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.sp, 10);
        assert_eq!(cpu.stack[11], PROGRAM_START_ADDRESS);
    }

    // TODO: CLS

    #[test]
    fn test_op_00ee_ret() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.sp = 10;
        cpu.stack[10] = 0xBEEF;

        execute_instruction(&mut cpu, &mut memory, 0x00ee);

        assert_eq!(cpu.pc, 0xBEEF + 2);
        assert_eq!(cpu.sp, 9);
    }

    #[test]
    fn test_op_1nnn_jp() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);

        execute_instruction(&mut cpu, &mut memory, 0x1DAD);

        assert_eq!(cpu.pc, 0x0DAD);
    }

    #[test]
    fn test_op_2nnn_call() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.sp = 10;

        execute_instruction(&mut cpu, &mut memory, 0x2DAD);

        assert_eq!(cpu.pc, 0x0DAD);
        assert_eq!(cpu.sp, 11);
        assert_eq!(cpu.stack[11], PROGRAM_START_ADDRESS);
    }

    #[test]
    fn test_op_3xkk_sevx() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x07;
        execute_instruction(&mut cpu, &mut memory, 0x3A06);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 1);

        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x07;
        execute_instruction(&mut cpu, &mut memory, 0x3A07);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 2);
    }

    #[test]
    fn test_op_4xkk_snevx() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x07;
        execute_instruction(&mut cpu, &mut memory, 0x4A06);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 2);

        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x07;
        execute_instruction(&mut cpu, &mut memory, 0x4A07);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 1);
    }

    #[test]
    fn test_op_5xy0_sevxvy() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x07;
        cpu.v[0xB] = 0x06;
        execute_instruction(&mut cpu, &mut memory, 0x5AB0);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 1);

        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x07;
        cpu.v[0xB] = 0x07;
        execute_instruction(&mut cpu, &mut memory, 0x5AB0);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 2);
    }

    #[test]
    fn test_op_6xkk_ldvx() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x06;

        execute_instruction(&mut cpu, &mut memory, 0x6A07);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0xA], 0x07);
    }

    #[test]
    fn test_op_7xkk_addvx() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x06;
        execute_instruction(&mut cpu, &mut memory, 0x7A01);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0xA], 0x07);

        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0xFF;
        execute_instruction(&mut cpu, &mut memory, 0x7A01);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0xA], 0x00);
    }
    #[test]
    fn test_op_8xy0() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x00;
        cpu.v[0xB] = 0x07;
        execute_instruction(&mut cpu, &mut memory, 0x8AB0);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0xA], 0x07);
        assert_eq!(cpu.v[0xB], 0x07);
    }

    #[test]
    fn test_op_8xy1_orvxvy() {
        // 0x0F or 0xF0 == 0xFF
        test_math(0x0F, 0xF0, 1, 0xFF, 0);
    }

    #[test]
    fn test_op_8xy2_andvxvy() {
        // 0x0F and 0xFF == 0x0F
        test_math(0x0F, 0xFF, 2, 0x0F, 0);
    }

    #[test]
    fn test_op_8xy3_xorvxvy() {
        // 0x0F xor 0xFF == 0xF0
        test_math(0x0F, 0xFF, 3, 0xF0, 0);
    }

    #[test]
    fn test_op_8xy4_addvxvy() {
        test_math(0x0F, 0x0F, 4, 0x1E, 0);
        test_math(0xFF, 0xFF, 4, 0xFE, 1);
    }

    #[test]
    fn test_op_8xy5_subvxvy() {
        test_math(0x0F, 0x01, 5, 0x0E, 1);
        test_math(0x0F, 0xFF, 5, 0x10, 0);
    }
    #[test]
    fn test_op_8x06_shrvx() {
        // 4 >> 1 == 2
        test_math(0x04, 0, 6, 0x02, 0);
        // 5 >> 1 == 2 with carry
        test_math(0x05, 0, 6, 0x02, 1);
    }

    #[test]
    fn test_op_8xy7_subnvxvy() {
        test_math(0x01, 0x0F, 7, 0x0E, 1);
        test_math(0xFF, 0x0F, 7, 0x10, 0);
    }

    #[test]
    fn test_op_8x0e_shlvx() {
        test_math(0b11000000, 0, 0x0e, 0b10000000, 1);
        test_math(0b00000111, 0, 0x0e, 0b00001110, 0);
    }

    #[test]
    fn test_op_9xy0_snevxvy() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x07;
        cpu.v[0xB] = 0x06;
        execute_instruction(&mut cpu, &mut memory, 0x9AB0);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 2);

        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0xA] = 0x07;
        cpu.v[0xB] = 0x07;
        execute_instruction(&mut cpu, &mut memory, 0x9AB0);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
    }

    #[test]
    fn test_op_annn_ldi() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);

        execute_instruction(&mut cpu, &mut memory, 0xaDAD);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.i, 0xDAD);
    }

    #[test]
    fn test_op_bnnn_jpv0() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0x0] = 0x07;

        execute_instruction(&mut cpu, &mut memory, 0xbDAD);

        assert_eq!(cpu.pc, 0xDAD + 0x07);
    }

    #[test]
    fn test_op_cxkk_randvx() {
        // TODO: Add test for non null value
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);

        execute_instruction(&mut cpu, &mut memory, 0xcA00);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0xA], 0);
    }

    #[test]
    fn test_op_dxyn_drwvxvyn() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0x8] = 0xA; // Set X position for drawing
        cpu.v[0x9] = 0xB; // Set Y position for drawing
        cpu.i = (PROGRAM_START_ADDRESS + 6) as u16;

        // Sprites to draw
        memory.write(PROGRAM_START_ADDRESS + 6, 0b11010000);
        memory.write(PROGRAM_START_ADDRESS + 7, 0b10100000);

        let mut display = instantiate_display();
        execute_instruction_with_display(&mut cpu, &mut memory, &mut display, 0xD892);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0xF], 0);

        assert_eq!(display.read_pixel(0xA + 0, 0xB + 0), true);
        assert_eq!(display.read_pixel(0xA + 1, 0xB + 0), true);
        assert_eq!(display.read_pixel(0xA + 2, 0xB + 0), false);
        assert_eq!(display.read_pixel(0xA + 3, 0xB + 0), true);
        assert_eq!(display.read_pixel(0xA + 4, 0xB + 0), false);
        assert_eq!(display.read_pixel(0xA + 5, 0xB + 0), false);
        assert_eq!(display.read_pixel(0xA + 6, 0xB + 0), false);
        assert_eq!(display.read_pixel(0xA + 8, 0xB + 0), false);
        assert_eq!(display.read_pixel(0xA + 8, 0xB + 0), false);

        assert_eq!(display.read_pixel(0xA + 0, 0xB + 1), true);
        assert_eq!(display.read_pixel(0xA + 1, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 2, 0xB + 1), true);
        assert_eq!(display.read_pixel(0xA + 3, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 4, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 5, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 6, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 8, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 8, 0xB + 1), false);
    }

    #[test]
    fn test_op_dxyn_drwvxvyn_erase() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        cpu.v[0x8] = 0xA; // Set X position for drawing
        cpu.v[0x9] = 0xB; // Set Y position for drawing
        cpu.i = (PROGRAM_START_ADDRESS + 6) as u16;

        // Sprites to draw
        memory.write(PROGRAM_START_ADDRESS + 6, 0b11010000);
        memory.write(PROGRAM_START_ADDRESS + 7, 0b10100000);

        // Initial display state
        let mut display = instantiate_display();
        display.write_pixel(0xA + 0, 0xB + 1, true);

        execute_instruction_with_display(&mut cpu, &mut memory, &mut display, 0xD892);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0xF], 1);

        assert_eq!(display.read_pixel(0xA + 0, 0xB + 0), true);
        assert_eq!(display.read_pixel(0xA + 1, 0xB + 0), true);
        assert_eq!(display.read_pixel(0xA + 2, 0xB + 0), false);
        assert_eq!(display.read_pixel(0xA + 3, 0xB + 0), true);
        assert_eq!(display.read_pixel(0xA + 4, 0xB + 0), false);
        assert_eq!(display.read_pixel(0xA + 5, 0xB + 0), false);
        assert_eq!(display.read_pixel(0xA + 6, 0xB + 0), false);
        assert_eq!(display.read_pixel(0xA + 7, 0xB + 0), false);

        assert_eq!(display.read_pixel(0xA + 0, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 1, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 2, 0xB + 1), true);
        assert_eq!(display.read_pixel(0xA + 3, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 4, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 5, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 6, 0xB + 1), false);
        assert_eq!(display.read_pixel(0xA + 7, 0xB + 1), false);
    }

    #[test]
    fn test_op_dxyn_drwvxvyn_wrap_horizontal() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        let x_position = crate::display::DISPLAY_WIDTH - 4;
        cpu.v[0x8] = x_position as u8; // Set X position for drawing
        cpu.v[0x9] = 0xB; // Set Y position for drawing
        cpu.i = (PROGRAM_START_ADDRESS + 6) as u16;

        // Sprites to draw
        memory.write(PROGRAM_START_ADDRESS + 6, 0b00101011);

        let mut display = instantiate_display();
        execute_instruction_with_display(&mut cpu, &mut memory, &mut display, 0xD891);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0xF], 0);

        assert_eq!(display.read_pixel(x_position + 0, 0xB), false);
        assert_eq!(display.read_pixel(x_position + 1, 0xB), false);
        assert_eq!(display.read_pixel(x_position + 2, 0xB), true);
        assert_eq!(display.read_pixel(x_position + 3, 0xB), false);
        assert_eq!(display.read_pixel(0 + 0, 0xB), true);
        assert_eq!(display.read_pixel(0 + 1, 0xB), false);
        assert_eq!(display.read_pixel(0 + 2, 0xB), true);
        assert_eq!(display.read_pixel(0 + 3, 0xB), true);
    }


    #[test]
    fn test_op_dxyn_drwvxvyn_wrap_vertical() {
        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        let y_position = crate::display::DISPLAY_HEIGHT - 1;
        cpu.v[0x8] = 0xA; // Set X position for drawing
        cpu.v[0x9] = y_position as u8; // Set Y position for drawing
        cpu.i = (PROGRAM_START_ADDRESS + 6) as u16;

        // Sprites to draw
        memory.write(PROGRAM_START_ADDRESS + 6, 0b00101011);
        memory.write(PROGRAM_START_ADDRESS + 7, 0b10100000);

        let mut display = instantiate_display();
        execute_instruction_with_display(&mut cpu, &mut memory, &mut display, 0xD892);

        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2);
        assert_eq!(cpu.v[0xF], 0);

        assert_eq!(display.read_pixel(0xA + 0, y_position), false);
        assert_eq!(display.read_pixel(0xA + 1, y_position), false);
        assert_eq!(display.read_pixel(0xA + 2, y_position), true);
        assert_eq!(display.read_pixel(0xA + 3, y_position), false);
        assert_eq!(display.read_pixel(0xA + 4, y_position), true);
        assert_eq!(display.read_pixel(0xA + 5, y_position), false);
        assert_eq!(display.read_pixel(0xA + 6, y_position), true);
        assert_eq!(display.read_pixel(0xA + 7, y_position), true);

        assert_eq!(display.read_pixel(0xA + 0, 0), true);
        assert_eq!(display.read_pixel(0xA + 1, 0), false);
        assert_eq!(display.read_pixel(0xA + 2, 0), true);
        assert_eq!(display.read_pixel(0xA + 3, 0), false);
        assert_eq!(display.read_pixel(0xA + 4, 0), false);
        assert_eq!(display.read_pixel(0xA + 5, 0), false);
        assert_eq!(display.read_pixel(0xA + 6, 0), false);
        assert_eq!(display.read_pixel(0xA + 7, 0), false);
    }

    #[test]
    fn test_op_ex9e_skpvx() {
        // Note: Keycode A is mapped to hex 0x7

        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        let mut keypad = instantiate_keypad();
        execute_instruction_with_keypad(&mut cpu, &mut memory, &mut keypad, 0xE79E);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 1);
        keypad.key_down(Keycode::S);
        execute_instruction_with_keypad(&mut cpu, &mut memory, &mut keypad, 0xE79E);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 1 + 2 * 1);

        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        let mut keypad = instantiate_keypad();
        keypad.key_down(Keycode::A);
        keypad.key_down(Keycode::X);
        keypad.key_up(Keycode::S);
        keypad.key_down(Keycode::A);
        execute_instruction_with_keypad(&mut cpu, &mut memory, &mut keypad, 0xE79E);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 2);
    }

    #[test]
    fn test_op_exa1_sknpvx() {
        // Note: Keycode A is mapped to hex 0x7

        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        let mut keypad = instantiate_keypad();
        execute_instruction_with_keypad(&mut cpu, &mut memory, &mut keypad, 0xE7A1);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 2);
        keypad.key_down(Keycode::S);
        execute_instruction_with_keypad(&mut cpu, &mut memory, &mut keypad, 0xE7A1);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 2 + 2 * 2);

        let mut memory = instantiate_memory();
        let mut cpu = instantiate_cpu(&mut memory);
        let mut keypad = instantiate_keypad();
        keypad.key_down(Keycode::A);
        keypad.key_down(Keycode::X);
        keypad.key_up(Keycode::S);
        keypad.key_down(Keycode::A);
        execute_instruction_with_keypad(&mut cpu, &mut memory, &mut keypad, 0xE7A1);
        assert_eq!(cpu.pc, PROGRAM_START_ADDRESS + 2 * 1);
    }

    // TODO: Implement tests
    // // LD Vx, DT
    // #[test]
    // fn test_op_fx07() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     cpu.delay_timer = 20;
    //     cpu.run_opcode(0xf507);
    //     assert_eq!(cpu.v[5], 20);
    //     assert_eq!(cpu.pc, NEXT_PC);
    // }
    // // LD Vx, K
    // #[test]
    // fn test_op_fx0a() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     cpu.run_opcode(0xf50a);
    //     assert_eq!(cpu.keypad_waiting, true);
    //     assert_eq!(cpu.keypad_register, 5);
    //     assert_eq!(cpu.pc, NEXT_PC);
    //     // Tick with no keypresses doesn't do anything
    //     cpu.tick([false; 16]);
    //     assert_eq!(cpu.keypad_waiting, true);
    //     assert_eq!(cpu.keypad_register, 5);
    //     assert_eq!(cpu.pc, NEXT_PC);
    //     // Tick with a keypress finishes wait and loads
    //     // first pressed key into vx
    //     cpu.tick([true; 16]);
    //     assert_eq!(cpu.keypad_waiting, false);
    //     assert_eq!(cpu.v[5], 0);
    //     assert_eq!(cpu.pc, NEXT_PC);
    // }
    // // LD DT, vX
    // #[test]
    // fn test_op_fx15() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     cpu.v[5] = 9;
    //     cpu.run_opcode(0xf515);
    //     assert_eq!(cpu.delay_timer, 9);
    //     assert_eq!(cpu.pc, NEXT_PC);
    // }
    // // LD ST, vX
    // #[test]
    // fn test_op_fx18() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     cpu.v[5] = 9;
    //     cpu.run_opcode(0xf518);
    //     assert_eq!(cpu.sound_timer, 9);
    //     assert_eq!(cpu.pc, NEXT_PC);
    // }
    // // ADD I, Vx
    // #[test]
    // fn test_op_fx1e() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     cpu.v[5] = 9;
    //     cpu.i = 9;
    //     cpu.run_opcode(0xf51e);
    //     assert_eq!(cpu.i, 18);
    //     assert_eq!(cpu.pc, NEXT_PC);
    // }
    // // LD F, Vx
    // #[test]
    // fn test_op_fx29() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     cpu.v[5] = 9;
    //     cpu.run_opcode(0xf529);
    //     assert_eq!(cpu.i, 5 * 9);
    //     assert_eq!(cpu.pc, NEXT_PC);
    // }
    // // LD B, Vx
    // #[test]
    // fn test_op_fx33() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     cpu.v[5] = 123;
    //     cpu.i = 1000;
    //     cpu.run_opcode(0xf533);
    //     assert_eq!(memory.cells[1000], 1);
    //     assert_eq!(memory.cells[1001], 2);
    //     assert_eq!(memory.cells[1002], 3);
    //     assert_eq!(cpu.pc, NEXT_PC);
    // }
    // // LD [I], Vx
    // #[test]
    // fn test_op_fx55() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     cpu.i = 1000;
    //     cpu.run_opcode(0xff55);
    //     for i in 0..16 {
    //         assert_eq!(memory.cells[1000 + i as usize], cpu.v[i]);
    //     }
    //     assert_eq!(cpu.pc, NEXT_PC);
    // }
    // // LD Vx, [I]
    // #[test]
    // fn test_op_fx65() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     for i in 0..16 as usize {
    //         memory.cells[1000 + i] = i as u8;
    //     }
    //     cpu.i = 1000;
    //     cpu.run_opcode(0xff65);
    //     for i in 0..16 as usize {
    //         assert_eq!(cpu.v[i], memory.cells[1000 + i]);
    //     }
    //     assert_eq!(cpu.pc, NEXT_PC);
    // }
    // #[test]
    // fn test_timers() {
    //     let mut memory = instantiate_memory();
    //     let mut cpu = instantiate_cpu(&mut memory);
    //     cpu.delay_timer = 200;
    //     cpu.sound_timer = 100;
    //     cpu.tick([false; 16]);
    //     assert_eq!(cpu.delay_timer, 199);
    //     assert_eq!(cpu.sound_timer, 99);
    // }
}
