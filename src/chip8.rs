use crate::components::{Chip8Components, MEM_SIZE, PC_START_ADDR};
use crate::instruction::{decode, Chip8Instruction};
use anyhow::anyhow;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const DEFAULT_FPS: u64 = 60;

#[derive(Default)]
pub struct Chip8Emulator {
    state: Chip8Components,
}

impl Chip8Emulator {
    pub fn run(&mut self, rom: PathBuf) -> anyhow::Result<()> {
        let target_fps = DEFAULT_FPS; // TODO: Make this configurable
        let frame_duration = Duration::from_nanos(1_000_000_000 / target_fps);

        self.load_rom(rom)?;
        loop {
            let frame_start = Instant::now();

            // Fetch and execute the next instruction
            let instruction = self.fetch_instruction()?;
            instruction.execute(&mut self.state)?;

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
        self.state.memory[PC_START_ADDR..PC_START_ADDR + rom_data.len()].copy_from_slice(&rom_data);
        Ok(())
    }

    fn fetch_instruction(&mut self) -> anyhow::Result<Box<dyn Chip8Instruction>> {
        if self.state.pc + 1 >= MEM_SIZE {
            return Err(anyhow!("Program counter out of bounds"));
        }
        let high_byte = self.state.memory[self.state.pc] as u16;
        let low_byte = self.state.memory[self.state.pc + 1] as u16;

        // Move the program counter to next instruction
        self.state.pc += 2;

        decode((high_byte << 8) | low_byte)
    }
}
