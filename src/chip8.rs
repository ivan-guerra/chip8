use crate::display::{Chip8Display, Chip8TerminalDisplay};
use anyhow::anyhow;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const MEM_SIZE: usize = 4096;
const PC_START_ADDR: usize = 0x200;
const FONT_ADDR: usize = 0x50;
const NUM_REGISTERS: usize = 16;
const NUM_KEYS: usize = 16;
const DEFAULT_FPS: u64 = 60;

pub struct Chip8Emulator {
    memory: [u8; MEM_SIZE],
    registers: [u8; NUM_REGISTERS],
    pc: usize,
    stack: Vec<usize>,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; NUM_KEYS],
    display: Box<dyn Chip8Display>,
}

impl Default for Chip8Emulator {
    fn default() -> Self {
        Chip8Emulator {
            memory: [0; MEM_SIZE],
            registers: [0; NUM_REGISTERS],
            pc: PC_START_ADDR,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; NUM_KEYS],
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

        emulator.memory[FONT_ADDR..FONT_ADDR + font_data.len()].copy_from_slice(&font_data);

        emulator
    }

    pub fn run(&mut self, rom: PathBuf) -> anyhow::Result<()> {
        let target_fps = DEFAULT_FPS; // TODO: Make this configurable
        let frame_duration = Duration::from_nanos(1_000_000_000 / target_fps);

        self.load_rom(rom)?;
        loop {
            let frame_start = Instant::now();

            // TODO: Add emulation cycle here

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }

    fn load_rom(&mut self, rom: PathBuf) -> anyhow::Result<()> {
        let rom_data = std::fs::read(rom)?;

        if rom_data.len() > MEM_SIZE - PC_START_ADDR {
            return Err(anyhow!("ROM too large to fit in memory"));
        }
        self.memory[PC_START_ADDR..PC_START_ADDR + rom_data.len()].copy_from_slice(&rom_data);
        Ok(())
    }
}
