extern crate rodio;

use self::rodio::Source;
use std::fs::File;
use std::io::BufReader;

pub trait SpeakerTrait {
    fn queue_beep(&mut self);
    fn flush_queue(&mut self);
    fn clear_queue(&mut self);
}

pub struct Speaker {
    play_beep: bool,
}

impl SpeakerTrait for Speaker {
    fn queue_beep(&mut self) {
        self.play_beep = true;
    }

    fn flush_queue(&mut self) {
        if self.play_beep {
            Speaker::play_sound("resources/beep.wav");

            self.play_beep = false;
        }
    }

    fn clear_queue(&mut self) {
        self.play_beep = false;
    }
}

impl Speaker {
    pub fn new() -> Speaker {
        println!("Initializing speaker");

        Speaker { play_beep: false }
    }

    fn play_sound(file_name: &str) {
        let device = rodio::default_output_device().unwrap();

        let file = File::open(file_name).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        rodio::play_raw(&device, source.convert_samples());
    }
}
