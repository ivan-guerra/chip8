use crate::components::Chip8Components;
use anyhow::anyhow;

pub struct DecodedInstruction {
    /// First nibble. Represents the operation code.
    opcode: u8,
    /// Second nibble. Used to look up one of the 16 registers.
    x: u8,
    /// Third nibble. Used to look up one of the 16 registers.
    y: u8,
    /// Fourth nibble. A 4-bit number.
    n: u8,
    /// The second byte (third and fourth nibbles). An 8-bit immediate number.
    nn: u8,
    /// The second, third, and fourth nibbles. A 12-bit immediate address.
    nnn: u16,
}

impl DecodedInstruction {
    pub fn new(raw: u16) -> Self {
        DecodedInstruction {
            opcode: (raw >> 12) as u8,
            x: ((raw >> 8) & 0x0F) as u8,
            y: ((raw >> 4) & 0x0F) as u8,
            n: (raw & 0x0F) as u8,
            nn: (raw & 0x00FF) as u8,
            nnn: (raw & 0x0FFF),
        }
    }
}

pub trait Chip8Instruction {
    fn execute(&self, state: &mut Chip8Components) -> anyhow::Result<()>;
}

pub fn decode(raw: u16) -> anyhow::Result<Box<dyn Chip8Instruction>> {
    let decoded = DecodedInstruction::new(raw);

    match decoded.opcode {
        0x0 => match decoded.nnn {
            0x0E0 => Ok(Box::new(ClearScreen)),
            _ => Err(anyhow!(
                "Unsupported request for execute machine language routine: {:#04X}",
                raw
            )),
        },
        0x1 => Ok(Box::new(Jump(decoded))),
        0x6 => Ok(Box::new(Set(decoded))),
        0x7 => Ok(Box::new(Add(decoded))),
        0xA => Ok(Box::new(SetIndex(decoded))),
        0xD => Ok(Box::new(Display(decoded))),
        _ => Err(anyhow!("Unknown opcode: {:#04X}", decoded.opcode)),
    }
}

pub struct ClearScreen;

impl Chip8Instruction for ClearScreen {
    fn execute(&self, state: &mut Chip8Components) -> anyhow::Result<()> {
        state.display.clear();
        Ok(())
    }
}

pub struct Jump(DecodedInstruction);

impl Chip8Instruction for Jump {
    fn execute(&self, state: &mut Chip8Components) -> anyhow::Result<()> {
        state.pc = self.0.nnn as usize;
        Ok(())
    }
}

pub struct Set(DecodedInstruction);

impl Chip8Instruction for Set {
    fn execute(&self, state: &mut Chip8Components) -> anyhow::Result<()> {
        state.registers[self.0.x as usize] = self.0.nn;
        Ok(())
    }
}

pub struct Add(DecodedInstruction);

impl Chip8Instruction for Add {
    fn execute(&self, state: &mut Chip8Components) -> anyhow::Result<()> {
        let reg_x = &mut state.registers[self.0.x as usize];
        *reg_x = reg_x.wrapping_add(self.0.nn);
        Ok(())
    }
}

pub struct SetIndex(DecodedInstruction);

impl Chip8Instruction for SetIndex {
    fn execute(&self, state: &mut Chip8Components) -> anyhow::Result<()> {
        state.index = self.0.nnn as usize;
        Ok(())
    }
}

pub struct Display(DecodedInstruction);

impl Chip8Instruction for Display {
    fn execute(&self, state: &mut Chip8Components) -> anyhow::Result<()> {
        let x = state.registers[self.0.x as usize] as usize;
        let y = state.registers[self.0.y as usize] as usize;
        let sprite = &state.memory[state.index..state.index + self.0.n as usize];

        if state.display.draw_sprite(x, y, sprite) {
            // If there was a collision, set the VF register
            state.registers[0xF] = 1;
        } else {
            state.registers[0xF] = 0;
        }
        Ok(())
    }
}
