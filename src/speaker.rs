extern crate rodio;

use std::fs::File;
use std::io::BufReader;
use self::rodio::Source;



pub struct Speaker {
    play_beep: bool
}

impl Speaker {

    // Constructors
    pub fn new() -> Speaker {
        println!("Initializing speaker");

        Speaker {
            play_beep: false
        }
    }

    // Methods
    pub fn queue_beep(&mut self) {
        self.play_beep = true;
    }

    pub fn flush_queue(&mut self) {
        if self.play_beep {
            Speaker::play_sound("resources/beep.wav");

            self.play_beep = false;
        }
    }

    pub fn clear_queue(&mut self) {
        self.play_beep = false;
    }


    fn play_sound(file_name: &str) {
        let device = rodio::default_output_device().unwrap();

        let file = File::open(file_name).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        rodio::play_raw(&device, source.convert_samples());
    }

}
