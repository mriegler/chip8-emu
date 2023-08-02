use std::{
    error::Error,
    io::{stdout, Write},
};

use crossterm::{cursor, style::Styler, QueueableCommand};

use crate::{log, util::get_bit};

pub type PixelArray = [[bool; 32]; 64];

pub struct Screen {
    pixels: PixelArray,
    changed_pixels: PixelArray,
    first_render: bool,
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            pixels: [[false; 32]; 64],
            changed_pixels: [[true; 32]; 64],
            first_render: true,
        }
    }
}

impl Screen {
    pub fn render_sprite(&mut self, x: usize, y: usize, source: &[u8]) -> bool {
        let mut collided = false;
        for (i, el) in source.iter().enumerate() {
            log(format!("sprite line {}: {:08b}", i, el));
            for j in 0..8 {
                let mut target_x = x + j;
                let mut target_y = y + i;

                if target_x > 63 {
                    target_x = target_x % 64;
                }
                if target_y > 32 {
                    target_y = target_y % 32;
                }

                let source_pixel = get_bit(*el, 7 - j);
                log(format!(
                    "val: {}, location x {} y {}",
                    source_pixel, target_x, target_y
                ));
                let current_collided = self.xor_pixel(target_x, target_y, source_pixel);
                collided = collided || current_collided;
            }
        }

        collided
    }

    pub fn render_pixels(&mut self) -> Result<(), Box<dyn Error>> {
        let mut stdout = stdout();
        for y in 0..32 {
            for x in 0..64 {
                let changed_pixel = &mut self.changed_pixels[x][y];
                if !self.first_render && !*changed_pixel {
                    continue;
                }

                stdout.queue(cursor::MoveTo(x as u16, y as u16))?;
                if self.pixels[x][y] {
                    log(format!("show pixel at x {} y {}", x, y));
                    print!("{}", "█");
                } else {
                    print!("{}", "█".hidden());
                };
                *changed_pixel = false;
            }
            stdout.queue(cursor::MoveToNextLine(1))?;
        }
        stdout.queue(cursor::MoveToNextLine(1))?;
        stdout.flush()?;
        self.first_render = false;
        Ok(())
    }

    pub fn clear_screen(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                self.pixels[x][y] = false;
                self.changed_pixels[x][y] = true;
            }
        }
    }

    fn get_pixel(&mut self, x: usize, y: usize) -> &mut bool {
        &mut self.pixels[x][y]
    }

    fn xor_pixel(&mut self, x: usize, y: usize, new_val: bool) -> bool {
        let original = self.get_pixel(x, y);
        let xored = new_val ^ *original;

        self.set_pixel(x, y, xored)
    }

    fn set_pixel(&mut self, x: usize, y: usize, new_val: bool) -> bool {
        let original = self.get_pixel(x, y);
        let collided = *original && !new_val;
        let changed = *original != new_val;

        *original = new_val;

        if changed {
            self.changed_pixels[x][y] = true;
        }

        collided
    }
}
