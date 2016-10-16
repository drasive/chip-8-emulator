extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired,AudioSpecWAV,AudioCVT};
use sdl2::rwops::RWops;

use std::thread;
use std::time::Duration;


struct Sound {
    data: Vec<u8>,
    volume: f32,
    position: usize,
}

impl AudioCallback for Sound {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        for next_sound_byte in out.iter_mut() {
            *next_sound_byte = (*self.data.get(self.position).unwrap_or(&0) as f32 * self.volume) as u8;
            self.position += 1;
        }
    }
}


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

    pub fn flush_queue(&mut self, sdl2_audio: &sdl2::AudioSubsystem) {
        if self.play_beep {
            let beep_sound = include_bytes!("../resources/beep.wav");
            Speaker::play_sound(sdl2_audio, beep_sound);

            self.play_beep = false;
        }
    }

    pub fn clear_queue(&mut self) {
        self.play_beep = false;
    }


    fn play_sound(sdl2_audio: &sdl2::AudioSubsystem, sound_bytes: &[u8]) {
        let specification = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None
        };

        let sound = sdl2_audio.open_playback(None, &specification, |spec| {
            let mut wav_rw = RWops::from_bytes(sound_bytes).unwrap();
            let wav = AudioSpecWAV::load_wav_rw(&mut wav_rw).unwrap();
            let converter = AudioCVT::new(wav.format, wav.channels, wav.freq, spec.format, spec.channels, spec.freq).unwrap();
            let data = converter.convert(wav.buffer().to_vec());

            Sound {
                data: data,
                volume: 0.25,
                position: 0,
            }
        }).unwrap();        
        sound.resume();

        // TODO: Don't block the execution threat while playing sound
        let sound_duration: f32 = sound_bytes.len() as f32 / (specification.freq.unwrap() as f32 * specification.channels.unwrap() as f32) * 1000.0;
        thread::sleep(Duration::from_millis(sound_duration as u64 + 50));
    }

}
