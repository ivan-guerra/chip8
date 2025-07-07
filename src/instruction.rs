use crate::state::{Address, ChipState, Register};
use anyhow::anyhow;

pub trait Instruction {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()>;
}

pub fn decode(raw: u16) -> anyhow::Result<Box<dyn Instruction>> {
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

struct DecodedInstruction {
    /// First nibble. Represents the operation code.
    opcode: u8,
    /// Second nibble. Used to look up one of the 16 registers.
    x: usize,
    /// Third nibble. Used to look up one of the 16 registers.
    y: usize,
    /// Fourth nibble. A 4-bit number.
    n: u8,
    /// The second byte (third and fourth nibbles). An 8-bit immediate number.
    nn: u8,
    /// The second, third, and fourth nibbles. A 12-bit immediate address.
    nnn: Address,
}
impl DecodedInstruction {
    fn new(raw: u16) -> Self {
        DecodedInstruction {
            opcode: (raw >> 12) as u8,
            x: ((raw >> 8) & 0x0F) as usize,
            y: ((raw >> 4) & 0x0F) as usize,
            n: (raw & 0x0F) as u8,
            nn: (raw & 0x00FF) as u8,
            nnn: (raw & 0x0FFF) as usize,
        }
    }
}

struct ClearScreen;
impl Instruction for ClearScreen {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        state.display.clear();
        Ok(())
    }
}

struct Jump(DecodedInstruction);
impl Instruction for Jump {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        state.pc = self.0.nnn;
        Ok(())
    }
}

struct Set(DecodedInstruction);
impl Instruction for Set {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        state
            .registers
            .write(Register::from_index(self.0.x)?, self.0.nn);
        Ok(())
    }
}

struct Add(DecodedInstruction);
impl Instruction for Add {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_x_val = state.registers.read(reg_x);
        state
            .registers
            .write(reg_x, reg_x_val.wrapping_add(self.0.nn));
        Ok(())
    }
}

struct SetIndex(DecodedInstruction);
impl Instruction for SetIndex {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        state.index = self.0.nnn;
        Ok(())
    }
}

struct Display(DecodedInstruction);
impl Instruction for Display {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let x = state.registers.read(Register::from_index(self.0.x)?);
        let y = state.registers.read(Register::from_index(self.0.y)?);
        let sprite = state.memory.read_sprite(state.index, self.0.n)?;

        if state
            .display
            .draw_sprite(usize::from(x), usize::from(y), sprite)
        {
            state.registers.write(Register::VF, 1);
        } else {
            state.registers.write(Register::VF, 0);
        }
        Ok(())
    }
}
