use sdl2::keyboard::Keycode;

pub trait KeypadTrait {
    fn get_key(&mut self, key: u8) -> bool;
    fn key_down(&mut self, keycode: Keycode);
    fn key_up(&mut self, keycode: Keycode);
    fn reset(&mut self);
}

pub struct Keypad {
    keys: [bool; 16], // 16 hexadecimal keys (0-9 and A-F)
}

impl KeypadTrait for Keypad {
    fn get_key(&mut self, key: u8) -> bool {
        self.keys[key as usize]
    }

    fn key_down(&mut self, keycode: Keycode) {
        let key = Keypad::map_key(keycode);
        if key.is_some() {
            self.keys[key.unwrap() as usize] = true;
        }
    }

    fn key_up(&mut self, keycode: Keycode) {
        let key = Keypad::map_key(keycode);
        if key.is_some() {
            self.keys[key.unwrap() as usize] = false;
        }
    }

    fn reset(&mut self) {
        self.keys = [false; 16]
    }
}

impl Keypad {
    pub fn new() -> Keypad {
        println!("Initializing keypad");

        Keypad { keys: [false; 16] }
    }

    fn map_key(keycode: Keycode) -> Option<u8> {
        match keycode {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),

            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),

            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),

            Keycode::Y | Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),

            _ => None,
        }
    }
}
