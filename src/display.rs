extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::Window;

#[cfg(test)]
use mockall::{automock, predicate::*};

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
const COLOR_ON: sdl2::pixels::Color = Color::RGB(109, 170, 44);
const COLOR_OFF: sdl2::pixels::Color = Color::RGB(2, 95, 95);

#[cfg_attr(test, automock)]
pub trait DisplayTrait {
    fn create_window(
        &self,
        sdl_video: &sdl2::VideoSubsystem,
        title_addition: &str,
    ) -> sdl2::video::Window;
    fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> u8;
    fn needs_redraw(&self) -> bool;
    fn draw(&mut self, renderer: &mut sdl2::render::Canvas<Window>);
    fn clear(&mut self);
}

pub trait DisplayDebugTrait {
    fn read_pixel(&self, x: usize, y: usize) -> bool;
    fn write_pixel(&mut self, x: usize, y: usize, value: bool);
}

pub struct Display {
    // General
    pixels: [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
    needs_redraw: bool,

    // Configuration
    display_scale: u8,
}

impl DisplayTrait for Display {
    fn create_window(
        &self,
        sdl_video: &sdl2::VideoSubsystem,
        title_addition: &str,
    ) -> sdl2::video::Window {
        let title = format!("Chip-8 Emulator ({})", title_addition);

        sdl_video
            .window(
                &title,
                DISPLAY_WIDTH as u32 * self.display_scale as u32,
                DISPLAY_HEIGHT as u32 * self.display_scale as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap()
    }

    fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> u8 {
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

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn draw(&mut self, renderer: &mut sdl2::render::Canvas<Window>) {
        for x in 0..DISPLAY_WIDTH {
            for y in 0..DISPLAY_HEIGHT {
                if self.pixels[x as usize][y as usize] {
                    renderer.set_draw_color(COLOR_ON);
                } else {
                    renderer.set_draw_color(COLOR_OFF);
                }

                renderer
                    .fill_rect(Rect::new(
                        x as i32 * self.display_scale as i32,
                        y as i32 * self.display_scale as i32,
                        self.display_scale as u32,
                        self.display_scale as u32,
                    ))
                    .unwrap();
            }
        }

        renderer.present();
        self.needs_redraw = false;
    }

    fn clear(&mut self) {
        self.pixels = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
    }
}

impl DisplayDebugTrait for Display {
    fn read_pixel(&self, x: usize, y: usize) -> bool {
        self.pixels[x][y]
    }

    fn write_pixel(&mut self, x: usize, y: usize, value: bool) {
        self.pixels[x][y] = value;
    }
}

impl Display {
    pub fn new(display_scale: u8) -> Display {
        println!("Initializing display");

        Display {
            // General
            pixels: [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
            needs_redraw: false,

            // Configuration
            display_scale: display_scale,
        }
    }

    fn get_bit(byte: u8, bit_index: u8) -> bool {
        byte & (0x80 >> bit_index) != 0
    }
}
