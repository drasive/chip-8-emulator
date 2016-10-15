use sdl2::keyboard::Keycode;


pub struct Keypad {
	keys: [bool; 16], // 16 hexadecimal keys (0-9 and A-F)
}


impl Keypad {

	// Constructors
    pub fn new() -> Keypad {
    	println!("Initializing keypad");

    	Keypad {
    		keys: [false; 16]
   		}
    }

    // Methods
    pub fn get_key(&mut self, key: u8) -> bool {
    	self.keys[key as usize]
    }
	

    pub fn key_down(&mut self, keycode: Keycode) {
    	let key = Keypad::map_key(keycode);
    	if key.is_some() {
    		self.keys[key.unwrap() as usize] = true;
    	}
    }

    pub fn key_up(&mut self, keycode: Keycode) {
    	let key = Keypad::map_key(keycode);
    	if key.is_some() {
    		self.keys[key.unwrap() as usize] = false;
    	}
    }


	pub fn reset(&mut self) {
		self.keys = [false; 16]
	}


    // Helpers
    fn map_key(keycode: Keycode) -> Option<u8> {
    	match keycode {
    		Keycode::Num1 => Some(0x1), // Key "1"
    		Keycode::Num2 => Some(0x2), // Key "2"
    		Keycode::Num3 => Some(0x3), // Key "3"
    		Keycode::Num4 => Some(0xC), // Key "C"

    		Keycode::Q =>    Some(0x4), // Key "4"
    		Keycode::W =>    Some(0x5), // Key "5"
    		Keycode::E =>    Some(0x6), // Key "6"
    		Keycode::R =>    Some(0xD), // Key "D"

    		Keycode::A =>    Some(0x7), // Key "7"
    		Keycode::S =>    Some(0x8), // Key "8"
    		Keycode::D =>    Some(0x9), // Key "9"
    		Keycode::F =>    Some(0xE), // Key "E"

    		Keycode::Y |
    		Keycode::Z =>    Some(0xA), // Key "A"
    		Keycode::X =>    Some(0x0), // Key "0"
    		Keycode::C =>    Some(0xB), // Key "B"
    		Keycode::V =>    Some(0xF), // Key "F"

    		_          =>    None
    	}
    }

}
