use crate::instruction::{decode, Instruction};
use crate::state::{ChipState, MEM_SIZE};
use anyhow::anyhow;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const DEFAULT_FPS: u64 = 60;

#[derive(Default)]
pub struct Emulator {
    state: ChipState,
}

impl Emulator {
    pub fn run(&mut self, rom: PathBuf) -> anyhow::Result<()> {
        let target_fps = DEFAULT_FPS; // TODO: Make this configurable
        let frame_duration = Duration::from_nanos(1_000_000_000 / target_fps);
        let rom_data = std::fs::read(rom)?;

        self.state.memory.load_rom(&rom_data)?;

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

    fn fetch_instruction(&mut self) -> anyhow::Result<Box<dyn Instruction>> {
        if self.state.pc + 1 >= MEM_SIZE {
            return Err(anyhow!("Program counter out of bounds"));
        }
        let high_byte = u16::from(self.state.memory.read(self.state.pc)?);
        let low_byte = u16::from(self.state.memory.read(self.state.pc + 1)?);

        // Move the program counter to next instruction
        self.state.pc += 2;

        decode((high_byte << 8) | low_byte)
    }
}
