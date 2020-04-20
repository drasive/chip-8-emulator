const MEMORY_SIZE: usize = 4096;

pub trait MemoryTrait {
    fn get_cells(&mut self) -> &mut [u8; MEMORY_SIZE]; // TODO: Allow range access?
    fn read(&self, index: usize) -> u8;
    fn write(&mut self, index: usize, value: u8);
    fn clear(&mut self);
    fn get_size(&self) -> usize;
    fn print_debug_info(&self);
}

pub struct Memory {
    cells: [u8; MEMORY_SIZE], // 8-bit memory. Public in order to allow batch access.
}

impl MemoryTrait for Memory {
    fn get_cells(&mut self) -> &mut [u8; MEMORY_SIZE] {
        &mut self.cells
    }

    fn read(&self, index: usize) -> u8 {
        self.cells[index]
    }

    fn write(&mut self, index: usize, value: u8) {
        self.cells[index] = value;
    }

    fn clear(&mut self) {
        self.cells = [0; MEMORY_SIZE];
    }

    fn get_size(&self) -> usize {
        MEMORY_SIZE
    }

    fn print_debug_info(&self) {
        println!("");

        for index in 0..MEMORY_SIZE {
            if index == 0 || (index > 1 && Memory::modulo(index, 16) == 0) {
                print!("0x{:03X} ", index);
            }

            print!("{:>02X} ", self.read(index));

            if index > 0 && Memory::modulo(index + 1, 16) == 0 {
                println!("");
            }
        }
    }
}

impl Memory {
    pub fn new() -> Memory {
        println!(
            "Initializing {memory_size} bytes of main memory",
            memory_size = MEMORY_SIZE
        );

        Memory {
            cells: [0; MEMORY_SIZE],
        }
    }

    fn modulo(n1: usize, n2: usize) -> usize {
        n1 - n2 * ((n1 / n2) as usize)
    }
}
