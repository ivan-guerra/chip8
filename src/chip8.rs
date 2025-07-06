use crate::display::{Chip8Display, Chip8TerminalDisplay, FontSprite};
use std::time::{Duration, Instant};

pub struct Chip8Emulator {
    memory: [u8; 4096],
    registers: [u8; 16],
    pc: usize,
    stack: Vec<usize>,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],
    display: Box<dyn Chip8Display>,
}

impl Default for Chip8Emulator {
    fn default() -> Self {
        Chip8Emulator {
            memory: [0; 4096],
            registers: [0; 16],
            pc: 0x200, // Program starts at 0x200
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
            display: Box::new(Chip8TerminalDisplay::new()),
        }
    }
}

impl Chip8Emulator {
    pub fn new() -> Self {
        let font_data = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        let mut emulator = Chip8Emulator::default();

        emulator.memory[0x50..0x50 + font_data.len()].copy_from_slice(&font_data);

        emulator
    }

    pub fn run(&mut self) {
        let target_fps = 60; // TODO: Make this configurable
        let frame_duration = Duration::from_nanos(1_000_000_000 / target_fps);

        loop {
            let frame_start = Instant::now();

            // TODO: Add emulation cycle here

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }
}
