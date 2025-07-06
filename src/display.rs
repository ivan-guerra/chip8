use bitvec::{array::BitArray, BitArr};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub trait Chip8Display {
    fn clear(&mut self);
    fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool;
}

pub struct Chip8TerminalDisplay {
    pixels: BitArr!(for DISPLAY_WIDTH * DISPLAY_HEIGHT),
}

impl Chip8TerminalDisplay {
    pub fn new() -> Self {
        Chip8TerminalDisplay {
            pixels: BitArray::ZERO,
        }
    }

    fn update_screen(&self) {
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let index = y * DISPLAY_WIDTH + x;
                if self.pixels[index] {
                    print!("█"); // Draw filled pixel
                } else {
                    print!(" "); // Draw empty pixel
                }
            }
            println!(); // New line after each row
        }
    }
}

impl Chip8Display for Chip8TerminalDisplay {
    fn clear(&mut self) {
        self.pixels.fill(false);
    }

    fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;

        for (row, &byte) in sprite.iter().enumerate() {
            for bit in 0..8 {
                let pixel_x = x + bit;
                let pixel_y = y + row;

                // Skip pixels that are outside screen boundaries
                if pixel_x >= DISPLAY_WIDTH || pixel_y >= DISPLAY_HEIGHT {
                    continue;
                }

                let index = pixel_y * DISPLAY_WIDTH + pixel_x;
                let current_pixel = self.pixels[index];

                let new_pixel = (byte >> (7 - bit)) & 1 == 1;
                if current_pixel && new_pixel {
                    collision = true; // Collision detected
                }

                self.pixels.set(index, current_pixel ^ new_pixel);
            }
        }

        self.update_screen(); // Draw the display after updating pixels
        collision
    }
}
