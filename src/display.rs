use bitvec::{array::BitArray, BitArr};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

#[derive(Clone, Copy)]
pub struct FontSprite([u8; 5]);

impl FontSprite {
    pub const ZERO: FontSprite = FontSprite([0xF0, 0x90, 0x90, 0x90, 0xF0]);
    pub const ONE: FontSprite = FontSprite([0x20, 0x60, 0x20, 0x20, 0x70]);
    pub const TWO: FontSprite = FontSprite([0xF0, 0x10, 0xF0, 0x80, 0xF0]);
    pub const THREE: FontSprite = FontSprite([0xF0, 0x10, 0xF0, 0x10, 0xF0]);
    pub const FOUR: FontSprite = FontSprite([0x90, 0x90, 0xF0, 0x10, 0x10]);
    pub const FIVE: FontSprite = FontSprite([0xF0, 0x80, 0xF0, 0x10, 0xF0]);
    pub const SIX: FontSprite = FontSprite([0xF0, 0x80, 0xF0, 0x90, 0xF0]);
    pub const SEVEN: FontSprite = FontSprite([0xF0, 0x10, 0x20, 0x40, 0x40]);
    pub const EIGHT: FontSprite = FontSprite([0xF0, 0x90, 0xF0, 0x90, 0xF0]);
    pub const NINE: FontSprite = FontSprite([0xF0, 0x90, 0xF0, 0x10, 0xF0]);
    pub const A: FontSprite = FontSprite([0xF0, 0x90, 0xF0, 0x90, 0x90]);
    pub const B: FontSprite = FontSprite([0xE0, 0x90, 0xE0, 0x90, 0xE0]);
    pub const C: FontSprite = FontSprite([0xF0, 0x80, 0x80, 0x80, 0xF0]);
    pub const D: FontSprite = FontSprite([0xE0, 0x90, 0x90, 0x90, 0xE0]);
    pub const E: FontSprite = FontSprite([0xF0, 0x80, 0xF0, 0x80, 0xF0]);
    pub const F: FontSprite = FontSprite([0xF0, 0x80, 0xF0, 0x80, 0x80]);

    pub fn get_font_sprite(digit: u8) -> Option<FontSprite> {
        match digit {
            0x0 => Some(Self::ZERO),
            0x1 => Some(Self::ONE),
            0x2 => Some(Self::TWO),
            0x3 => Some(Self::THREE),
            0x4 => Some(Self::FOUR),
            0x5 => Some(Self::FIVE),
            0x6 => Some(Self::SIX),
            0x7 => Some(Self::SEVEN),
            0x8 => Some(Self::EIGHT),
            0x9 => Some(Self::NINE),
            0xA => Some(Self::A),
            0xB => Some(Self::B),
            0xC => Some(Self::C),
            0xD => Some(Self::D),
            0xE => Some(Self::E),
            0xF => Some(Self::F),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8; 5] {
        &self.0
    }
}

pub trait Chip8Display {
    fn clear(&mut self);
    fn draw_sprite(&mut self, x: usize, y: usize, sprite: &FontSprite) -> bool;
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

    fn draw_sprite(&mut self, x: usize, y: usize, sprite: &FontSprite) -> bool {
        let mut collision = false;

        for (row, &byte) in sprite.as_bytes().iter().enumerate() {
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
