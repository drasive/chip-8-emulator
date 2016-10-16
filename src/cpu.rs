extern crate rand;
extern crate sdl2;

use std::io::{Read, Write, BufWriter, Error};

use memory::Memory;
use keypad::Keypad;
use display::Display;
use speaker::Speaker;


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


pub struct Cpu {
    // Program
    opcode: u16, // current opcode (two 8-bit values)
    pc: usize, // 16-bit program counter

    // Registers
    v: [u8; 16], // 16 8-bit general purpose registers
    i: u16, // 16-bit register for storing memory adresses

    delay_timer: u8, // 8-bit delay timer (decremented at 60 Hz)
    delay_timer_f: f32, // float representation of delay_timer
    sound_timer: u8, // 8-bit sound timer (decremented at 60 Hz)
    sound_timer_f: f32, // float representation of sound_timer

    // Stack
    stack: [usize; 16], // 16 values to store return values of subroutines
    sp: u8, // 8-bit register that points to the topmost level of the stack

    // Configuration
    clock_rate: f32,
    ignore_unknown_instructions: bool,
    program_address: usize
}


impl Cpu {

    // Constructors
    pub fn new(clock_rate: f32, ignore_unknown_instructions: bool, program_address: usize) -> Cpu {
        println!("Initializing processor with {clock_rate} Hz", clock_rate = clock_rate);

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
            program_address: program_address
        }
    }

    // Methods
    pub fn load_rom(&mut self, memory: &mut Memory, reader: &mut Read) -> Result<usize, Error> {
        // Clear memory
        memory.clear();

        // Copy font to memory at 0x000
        if FONT_BYTES > memory.get_size() {
            panic!("Font size ({font_bytes} bytes) is larger than available memory ({memory_size} bytes)",
                font_bytes = FONT_BYTES, memory_size = memory.get_size());
        }

        {
            println!("Copying font ({font_bytes} bytes) to memory at 0x000", font_bytes = FONT_BYTES);
            let mut memory_stream = BufWriter::new(&mut memory.cells[0..FONT_BYTES]);
            memory_stream.write_all(FONT.as_ref()).unwrap();
        }

        // Read ROM
        println!("Reading ROM");
        let mut rom = Vec::new();
        try!(reader.read_to_end(&mut rom));

        // Copy ROM into memory
        if rom.len() < 2 {
            panic!("ROM does not contain any instructions");
        }
        else if rom.len() > memory.get_size() - self.program_address {
            panic!("ROM size ({rom_size} bytes) is larger than available program memory ({available_memory}) bytes)",
                   rom_size = rom.len(), available_memory = memory.get_size() - self.program_address);
        }
        
        {
            println!("Copying ROM ({rom_size} bytes) to memory at 0x{program_start:X}", rom_size = rom.len(), program_start = self.program_address);
            let mut memory_stream = BufWriter::new(&mut memory.cells[self.program_address..(self.program_address + rom.len())]);
            try!(memory_stream.write_all(rom.as_ref()));
        }


        self.pc = self.program_address;
        return Ok(rom.len());
    }

    pub fn step(
        &mut self, delta_time: f32,
        memory: &mut Memory, keypad: &mut Keypad, display: &mut Display, speaker: &mut Speaker,
        debug_cpu: bool, debug_memory: bool) {

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

    pub fn get_clock_rate(&self) -> f32 {
        self.clock_rate
    }

    pub fn print_debug_info(&self) {
        let opname = Cpu::get_opname(&self.opcode);
        println!("Op: 0x{:X} {}, PC: {}, I: 0x{:X}, DT: {}, ST: {}",
                 self.opcode, opname, self.pc, self.i, self.delay_timer, self.sound_timer);

        println!("Registers: {:?}", self.v);
        println!("Stack: {:?}", self.stack);
    }


    fn execute_instruction(&mut self, memory: &mut Memory, keypad: &mut Keypad, display: &mut Display) {
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

                self.pc = self.stack[self.sp as usize ] as usize;
                self.sp -= 1;

                self.pc += 2;
            }

            (0x1, _, _, _) => {
                // 1nnn - JP addr; Jump to location nnn.
                // The interpreter sets the program counter to nnn.

                self.pc = self.op_0fff();
            }
            (0x2, _, _, _) => {
                // 2nnn - CALL addr; Call subroutine at nnn.
                // The interpreter increments the stack pointer, then puts the current PC on the top of the stack.
                // The PC is then set to nnn.

                self.sp += 1;
                self.stack[self.sp as usize] = self.pc as usize;
                self.pc = self.op_0fff();
            }
            (0x3, x, _, _) => {
                // 3xkk - SE Vx, byte; Skip next instruction if Vx = kk.
                // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.

                if self.v[x] == self.op_00ff() as u8 {
                    self.pc += 2 * 2;
                }
                else {
                    self.pc += 2;
                }
            }
            (0x4, x, _, _) => {
                // 4xkk - SNE Vx, byte; Skip next instruction if Vx != kk.
                // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.

                if self.v[x] != self.op_00ff() as u8 {
                    self.pc += 2 * 2;
                }
                else {
                    self.pc += 2;
                }
            }
            (0x5, x, y, 0x0) => {
                // 5xy0 - SE Vx, Vy; Skip next instruction if Vx = Vy.
                // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.

                if self.v[x] == self.v[y] {
                    self.pc += 2 * 2;
                }
                else {
                    self.pc += 2;
                }
            }
            (0x6, x, _, _) => {
                // 6xkk - LD Vx, byte; Set Vx = kk.
                // The interpreter puts the value kk into register Vx.

                self.v[x] = self.op_00ff() as u8;

                self.pc += 2;
            }
            (0x7, x, _, _) => {
                // 7xkk - ADD Vx, byte; Set Vx = Vx + kk.
                // Adds the value kk to the value of register Vx, then stores the result in Vx. 

                self.v[x] = self.v[x].wrapping_add(self.op_00ff() as u8);

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
                self.v[0xF] = if flag { 0 } else { 1 };

                self.pc += 2;
            }
            (0x8, x, y, 0x5) => {
                // 8xy5 - SUB Vx, Vy; Set Vx = Vx - Vy, set VF = NOT borrow.
                // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.

                let (result, flag) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = result as u8;
                self.v[0xF] = if flag { 0 } else { 1 };

                self.pc += 2;
            }
            (0x8, x, _, 0x6) => {
                // 8xy6 - SHR Vx {, Vy}; Set Vx = Vx SHR 1.
                // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.

                self.v[x] >>= 1;
                self.v[0xF] = self.v[x] & 0x1;                

                self.pc += 2;
            }
            (0x8, x, y, 0x7) => {
                // 8xy7 - SUBN Vx, Vy; Set Vx = Vy - Vx, set VF = NOT borrow.
                // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.

                let (result, flag) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = result as u8;
                self.v[0xF] = if flag { 0 } else { 1 };

                self.pc += 2;
            }
            (0x8, x, _, 0xE) => {
                // 8xyE - SHL Vx {, Vy}; Set Vx = Vx SHL 1.
                // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.

                self.v[x] <<= 1;
                self.v[0xF] = (self.v[x] >> 7) & 0x1;

                self.pc += 2;
            }
            (0x9, x, y, 0x0) => {
                // 9xy0 - SNE Vx, Vy; Skip next instruction if Vx != Vy.
                // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.

                if self.v[x] != self.v[y] {
                    self.pc += 2 * 2;
                }
                else {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                // Annn - LD I, addr; Set I = nnn.
                // The value of register I is set to nnn.

                self.i = self.op_0fff() as u16;

                self.pc += 2;
            }
            (0xB, _, _, _) => {
                // Bnnn - JP V0, addr; Jump to location nnn + V0.
                // The program counter is set to nnn plus the value of V0.

                self.pc = self.op_0fff() + self.v[0x0] as usize;
            }
            (0xC, x, _, _) => {
                // Cxkk - RND Vx, byte; Set Vx = random byte AND kk.
                // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
                // The results are stored in Vx.

                self.v[x] = self.op_00ff() as u8 & rand::random::<u8>();

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
                self.v[0xF] = display.draw_sprite(self.v[x] as usize, self.v[y] as usize, &memory.cells[start..end]);

                self.pc += 2;
            }
            (0xE, x, 0x9, 0xE) => {
                // Ex9E - SKP Vx; Skip next instruction if key with the value of Vx is pressed.
                // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position,
                // PC is increased by 2.

                if keypad.get_key(x as u8) {
                    self.pc += 2 * 2;
                }
                else {
                    self.pc += 2;
                }
            }
            (0xE, x, 0xA, 0x1) => {
                // ExA1 - SKNP Vx; Skip next instruction if key with the value of Vx is not pressed.
                // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position,
                // PC is increased by 2.

                if !keypad.get_key(x as u8) {
                    self.pc += 2 * 2;
                }
                else {
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

                for index in 0..15 {
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
            _  => {
                // opcode "SYS" is intentionally not implemented
                if self.ignore_unknown_instructions {
                    println!("instruction not implemented. opcode: {opcode}, program counter: {program_counter}",
                             opcode = self.opcode, program_counter = self.pc);
                }
                else {
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

    fn update_sound_timer(&mut self, delta_time: f32, speaker: &mut Speaker) {
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
    

    // Helpers
    fn op_00ff(&mut self) -> usize {
        self.opcode as usize & 0x00FF 
    }

    fn op_0fff(&mut self) -> usize {
        self.opcode as usize & 0x0FFF
    }


    fn get_opname(opcode: & u16) -> &str {
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
            _  => "?"
        }
    }
    
}
