const MEMORY_SIZE: usize = 4096;


pub struct Memory {
    pub cells: [u8; MEMORY_SIZE] // 8-bit memory. Public in order to allow batch access.
}


impl Memory {

    // Constructors
    pub fn new() -> Memory {
        println!("Initializing {memory_size} bytes of main memory", memory_size = MEMORY_SIZE);

        Memory {
            cells: [0; MEMORY_SIZE]
        }
    }


    // Methods
    pub fn read(&self, index: usize) -> u8 {
        self.cells[index]
    }

    pub fn write(&mut self, index: usize, value: u8) {
        self.cells[index] = value;
    }

    pub fn clear(&mut self) {
        self.cells = [0; MEMORY_SIZE];
    }

    
    pub fn get_size(&self) -> usize {
        MEMORY_SIZE    
    }

    pub fn print_debug_info(&self) {
        for index in 0..MEMORY_SIZE {
            if Memory::modulo(index, 16) == 0 {
                print!("0x{:X} ", index);
            }

            print!("{:X} ", self.read(index));

            if Memory::modulo(index, 16) == 0 {
                println!("");
            }
        }
    }


    // Helpers
    fn modulo(n1: usize, n2: usize) -> usize {
        n1 - n2 * ((n1 / n2) as usize)
    }

}
