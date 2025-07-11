use crate::display::{Display, TerminalDisplay};
use anyhow::anyhow;

pub type Timer = u8;
pub type Address = usize;
pub type CallStack = Vec<Address>;

pub const MEM_SIZE: usize = 4096;
pub const FONT_ADDR: Address = 0x50;
pub const FONT_WIDTH: usize = 4;
pub const FONT_HEIGHT: usize = 5;
pub const PC_START_ADDR: Address = 0x200;
pub const NUM_KEYS: usize = 16;
pub const NUM_REGISTERS: usize = 16;

pub struct Memory {
    data: [u8; MEM_SIZE],
}
impl Memory {
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
        let data = {
            let mut data = [0; MEM_SIZE];
            data[FONT_ADDR..FONT_ADDR + font_data.len()].copy_from_slice(&font_data);
            data
        };

        Memory { data }
    }

    pub fn read(&self, addr: Address) -> anyhow::Result<u8> {
        if addr >= MEM_SIZE {
            return Err(anyhow!("Memory read out of bounds: {}", addr));
        }
        Ok(self.data[addr])
    }

    pub fn write(&mut self, addr: Address, value: u8) -> anyhow::Result<()> {
        if addr >= MEM_SIZE {
            return Err(anyhow!("Memory write out of bounds: {}", addr));
        }
        self.data[addr] = value;
        Ok(())
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> anyhow::Result<()> {
        if rom.len() > MEM_SIZE - PC_START_ADDR {
            return Err(anyhow!("ROM too large to fit in memory"));
        }
        self.data[PC_START_ADDR..PC_START_ADDR + rom.len()].copy_from_slice(rom);
        Ok(())
    }

    pub fn read_sprite(&self, index: Address, rows: u8) -> anyhow::Result<&[u8]> {
        let sprite_slice = index..index + rows as usize;

        if sprite_slice.end > MEM_SIZE {
            return Err(anyhow!("Sprite data out of bounds"));
        }
        Ok(&self.data[sprite_slice])
    }
}

#[derive(Copy, Clone)]
pub enum Register {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
}
impl Register {
    pub fn from_index(value: usize) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Register::V0),
            1 => Ok(Register::V1),
            2 => Ok(Register::V2),
            3 => Ok(Register::V3),
            4 => Ok(Register::V4),
            5 => Ok(Register::V5),
            6 => Ok(Register::V6),
            7 => Ok(Register::V7),
            8 => Ok(Register::V8),
            9 => Ok(Register::V9),
            10 => Ok(Register::VA),
            11 => Ok(Register::VB),
            12 => Ok(Register::VC),
            13 => Ok(Register::VD),
            14 => Ok(Register::VE),
            15 => Ok(Register::VF),
            _ => Err(anyhow!("Invalid register index: {}", value)),
        }
    }
}

pub struct RegisterBank {
    registers: [u8; NUM_REGISTERS],
}
impl RegisterBank {
    pub fn new() -> Self {
        RegisterBank {
            registers: [0; NUM_REGISTERS],
        }
    }

    pub fn read(&self, reg: Register) -> u8 {
        self.registers[reg as usize]
    }

    pub fn write(&mut self, reg: Register, value: u8) {
        self.registers[reg as usize] = value;
    }
}

pub struct ChipState {
    pub memory: Memory,
    pub registers: RegisterBank,
    pub pc: Address,
    pub index: Address,
    pub stack: CallStack,
    pub delay_timer: Timer,
    pub sound_timer: Timer,
    pub keypad: [bool; NUM_KEYS],
    pub display: Box<dyn Display>,
}
impl Default for ChipState {
    fn default() -> Self {
        ChipState {
            memory: Memory::new(),
            registers: RegisterBank::new(),
            pc: PC_START_ADDR,
            index: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; NUM_KEYS],
            display: Box::new(TerminalDisplay::new()), // TODO: Make this configurable
        }
    }
}
