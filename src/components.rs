use crate::display::{Chip8Display, Chip8TerminalDisplay};

pub const MEM_SIZE: usize = 4096;
pub const FONT_ADDR: usize = 0x50;
pub const PC_START_ADDR: usize = 0x200;
pub const NUM_KEYS: usize = 16;
pub const NUM_REGISTERS: usize = 16;

pub struct Chip8Components {
    pub memory: [u8; MEM_SIZE],
    pub registers: [u8; NUM_REGISTERS],
    pub pc: usize,
    pub index: usize,
    pub stack: Vec<usize>,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keypad: [bool; NUM_KEYS],
    pub display: Box<dyn Chip8Display>,
}

impl Default for Chip8Components {
    fn default() -> Self {
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
        let memory = {
            let mut mem = [0; MEM_SIZE];
            mem[FONT_ADDR..FONT_ADDR + font_data.len()].copy_from_slice(&font_data);
            mem
        };

        Chip8Components {
            memory,
            registers: [0; NUM_REGISTERS],
            pc: PC_START_ADDR,
            index: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; NUM_KEYS],
            display: Box::new(Chip8TerminalDisplay::new()), // TODO: Make this configurable
        }
    }
}
