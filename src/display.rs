extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::pixels::Color;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;


pub struct Display {
	// General
	pixels: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],

	// Objects
	ctx: sdl2::Sdl,
	video_ctx: sdl2::VideoSubsystem,
	display: sdl2::video::Window,

	// Configuration
	display_scale: u8
}


impl Display {

	// Constructors
    pub fn new(display_scale: u8) -> Display {
    	println!("Initializing display");

		let ctx = sdl2::init().unwrap();
    	let ctx_video = ctx.video().unwrap();
    	let display = ctx_video.window("Chip-8 Emulator"
    		,SCREEN_WIDTH as u32  * display_scale as u32, SCREEN_HEIGHT as u32 * display_scale as u32)
    		.position_centered().opengl().build().unwrap();

    	Display {
    		// General
    		pixels: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],

    		// Objects
    		ctx: ctx,
    		video_ctx: ctx_video,
    		display: display,

    		// Configuration
			display_scale: display_scale
    	}
    }

    // Methods
    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> u8 {
    	let mut collision = 0;

    	for row in 0..sprite.len() as usize {
    		for column in 0..8 as usize {
				let xp = (x + column) % SCREEN_WIDTH as usize;
				let yp = (y + row) % SCREEN_HEIGHT as usize;

				if Display::get_bit(sprite[row], column as u8) {
					let previous_state = self.pixels[yp][xp];

					self.pixels[yp][xp] = true;

					if previous_state && !self.pixels[yp][xp] {
						collision = 1;
					}
				}
    		}
    	}

    	collision
    }

    pub fn draw(&mut self) {
		let mut output: String = "".to_owned();

		for row in 0..SCREEN_HEIGHT - 1 {
			for column in 0..SCREEN_WIDTH - 1 {
				output.push_str(if self.pixels[row as usize][column as usize] { "XX" } else { "__" });
			}

			output.push_str("\n");
		}

		println!("{}", output);


		// TODO: Implement real renderer
		/*let mut renderer = self.display.renderer().build().unwrap();
		
    	for x in 0..SCREEN_WIDTH {
    		for y in 0..SCREEN_HEIGHT {
    			let color = if self.pixels[x as usize][y as usize] == 0 { 0 } else { 255 };
    			renderer.set_draw_color(Color::RGB(color, color, color));

    			renderer.draw_rect(Rect::new(x as i32, y as i32, self.display_scale as u32, self.display_scale as u32));
    		}
    	}

    	renderer.clear();
    	renderer.present();
	    //self.display.show();*/
    }

    pub fn clear(&mut self) {
    	self.pixels = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
    }


	// Helpers
	fn get_bit(byte: u8, bit_index: u8) -> bool {
		byte & (0x80 >> bit_index) != 0
	}

}
