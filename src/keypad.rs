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
    		Keycode::Num1 => Some( 0), // Key "1"
    		Keycode::Num2 => Some( 1), // Key "2"
    		Keycode::Num3 => Some( 2), // Key "3"
    		Keycode::Num4 => Some( 3), // Key "C"

    		Keycode::Q =>    Some( 4), // Key "4"
    		Keycode::W =>    Some( 5), // Key "5"
    		Keycode::E =>    Some( 6), // Key "6"
    		Keycode::R =>    Some( 7), // Key "D"

    		Keycode::A =>    Some( 8), // Key "7"
    		Keycode::S =>    Some( 9), // Key "8"
    		Keycode::D =>    Some(10), // Key "9"
    		Keycode::F =>    Some(11), // Key "E"

    		Keycode::Y |
    		Keycode::Z =>    Some(12), // Key "A"
    		Keycode::X =>    Some(13), // Key "0"
    		Keycode::C =>    Some(14), // Key "B"
    		Keycode::V =>    Some(15), // Key "F"

    		_          =>    None
    	}
    }

}
