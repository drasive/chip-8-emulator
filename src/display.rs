extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::pixels::Color;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;


pub struct Display {
	// General
	pixels: [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
	needs_redraw: bool,

	// Configuration
	display_scale: u8
}


impl Display {

	// Constructors
    pub fn new(display_scale: u8) -> Display {
    	println!("Initializing display");

    	Display {
    		// General
    		pixels: [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
			needs_redraw: false,

    		// Configuration
			display_scale: display_scale
    	}
    }


    // Methods
	pub fn create_window(&self, sdl_video: & sdl2::VideoSubsystem) -> sdl2::video::Window {
		sdl_video.window(
			"Chip-8 Emulator",
			DISPLAY_WIDTH as u32 * self.display_scale as u32,
			DISPLAY_HEIGHT as u32 * self.display_scale as u32)
			.position_centered().opengl().build().unwrap()
	}

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> u8 {
		self.needs_redraw = true;
    	let mut collision = 0;

    	for row in 0..sprite.len() as usize {
    		for column in 0..8 as usize {
				let xp = (x + column) % DISPLAY_WIDTH as usize;
				let yp = (y + row) % DISPLAY_HEIGHT as usize;

				let previous_state = self.pixels[xp][yp];
				self.pixels[xp][yp] ^= Display::get_bit(sprite[row], column as u8);
				if previous_state && !self.pixels[xp][yp] {
					collision = 1;
				}
    		}
    	}

    	collision
    }

    pub fn draw(&mut self, renderer: &mut sdl2::render::Renderer) {
    	for x in 0..DISPLAY_WIDTH {
    		for y in 0..DISPLAY_HEIGHT {
    			if self.pixels[x as usize][y as usize] { 
					renderer.set_draw_color(Color::RGB(109, 170, 44));
				} else {
					renderer.set_draw_color(Color::RGB(2, 95, 95));
				}

    			renderer.fill_rect(Rect::new(
					x as i32 * self.display_scale as i32 , y as i32 * self.display_scale as i32 ,
					self.display_scale as u32, self.display_scale as u32)).unwrap();
    		}
    	}

    	renderer.present();
		self.needs_redraw = false;
    }

    pub fn clear(&mut self) {
    	self.pixels = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
    }


	pub fn needs_redraw(&self) -> bool {
		self.needs_redraw
	}


	// Helpers
	fn get_bit(byte: u8, bit_index: u8) -> bool {
		byte & (0x80 >> bit_index) != 0
	}

}
