use crate::state::{Address, ChipState, Register, FONT_ADDR, FONT_HEIGHT};
use anyhow::anyhow;

pub trait Instruction {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()>;
}

pub fn decode(raw: u16) -> anyhow::Result<Box<dyn Instruction>> {
    let decoded = DecodedInstruction::new(raw);

    match decoded.opcode {
        0x0 => match decoded.nnn {
            0x0E0 => Ok(Box::new(ClearScreen)),
            0x0EE => Ok(Box::new(SubroutineReturn)),
            _ => Err(anyhow!(
                "Unsupported request for execute machine language routine: {:#04X}",
                raw
            )),
        },
        0x1 => Ok(Box::new(Jump(decoded))),
        0x2 => Ok(Box::new(SubroutineCall(decoded))),
        0x3 => Ok(Box::new(JumpEqX(decoded))),
        0x4 => Ok(Box::new(JumpNeqX(decoded))),
        0x5 => Ok(Box::new(JumpXEqY(decoded))),
        0x6 => Ok(Box::new(SetImmediate(decoded))),
        0x7 => Ok(Box::new(Add(decoded))),
        0x8 => match decoded.n {
            0x0 => Ok(Box::new(SetXToY(decoded))),
            0x1 => Ok(Box::new(BinaryOr(decoded))),
            0x2 => Ok(Box::new(BinaryAnd(decoded))),
            0x3 => Ok(Box::new(LogicalXor(decoded))),
            0x4 => Ok(Box::new(BinaryAdd(decoded))),
            0x5 => Ok(Box::new(SubtractYFromX(decoded))),
            0x6 => Ok(Box::new(RightShift(decoded))),
            0x8 => Ok(Box::new(SubtractXFromY(decoded))),
            0xE => Ok(Box::new(LeftShift(decoded))),
            _ => Err(anyhow!("Unsupported opcode for 0x8: {:#04X}", raw)),
        },
        0x9 => Ok(Box::new(JumpXNeqY(decoded))),
        0xA => Ok(Box::new(SetIndex(decoded))),
        0xB => Ok(Box::new(JumpWithOffset(decoded))),
        0xC => Ok(Box::new(Random(decoded))),
        0xD => Ok(Box::new(Display(decoded))),
        // TODO: 0xE* missing skip if key instructions
        0xF => match decoded.nn {
            0x07 => Ok(Box::new(SetVxFromTimer(decoded))),
            0x15 => Ok(Box::new(SetDelayTimer(decoded))),
            0x18 => Ok(Box::new(SetSoundTimer(decoded))),
            0x1E => Ok(Box::new(AddToIndex(decoded))),
            0x29 => Ok(Box::new(FontChar(decoded))),
            0x33 => Ok(Box::new(BinaryCodedDecimal(decoded))),
            0x55 => Ok(Box::new(Store(decoded))),
            0x65 => Ok(Box::new(Load(decoded))),
            // TODO: 0x0A missing get key instruction
            _ => Err(anyhow!("Unsupported opcode for 0xF: {:#04X}", raw)),
        },
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

struct SubroutineCall(DecodedInstruction);
impl Instruction for SubroutineCall {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        state.stack.push(state.pc);
        state.pc = self.0.nnn;
        Ok(())
    }
}

struct SubroutineReturn;
impl Instruction for SubroutineReturn {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        if let Some(return_address) = state.stack.pop() {
            state.pc = return_address;
            Ok(())
        } else {
            Err(anyhow!("Stack underflow: No return address available"))
        }
    }
}

struct JumpEqX(DecodedInstruction);
impl Instruction for JumpEqX {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        if state.registers.read(reg_x) == self.0.nn {
            state.pc += 2;
        }
        Ok(())
    }
}

struct JumpNeqX(DecodedInstruction);
impl Instruction for JumpNeqX {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        if state.registers.read(reg_x) != self.0.nn {
            state.pc += 2;
        }
        Ok(())
    }
}

struct JumpXEqY(DecodedInstruction);
impl Instruction for JumpXEqY {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        if state.registers.read(reg_x) == state.registers.read(reg_y) {
            state.pc += 2;
        }
        Ok(())
    }
}

struct JumpXNeqY(DecodedInstruction);
impl Instruction for JumpXNeqY {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        if state.registers.read(reg_x) != state.registers.read(reg_y) {
            state.pc += 2;
        }
        Ok(())
    }
}

struct SetImmediate(DecodedInstruction);
impl Instruction for SetImmediate {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        state.registers.write(reg_x, self.0.nn);
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

struct SetXToY(DecodedInstruction);
impl Instruction for SetXToY {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_y = state.registers.read(reg_y);
        state.registers.write(reg_x, value_y);
        Ok(())
    }
}

struct BinaryOr(DecodedInstruction);
impl Instruction for BinaryOr {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);
        state.registers.write(reg_x, value_x | value_y);
        Ok(())
    }
}

struct BinaryAnd(DecodedInstruction);
impl Instruction for BinaryAnd {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);
        state.registers.write(reg_x, value_x & value_y);
        Ok(())
    }
}

struct LogicalXor(DecodedInstruction);
impl Instruction for LogicalXor {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);
        state.registers.write(reg_x, value_x ^ value_y);
        Ok(())
    }
}

struct BinaryAdd(DecodedInstruction);
impl Instruction for BinaryAdd {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);
        let sum = value_x.wrapping_add(value_y);
        state.registers.write(reg_x, sum);

        // Set VF to 1 if there was no overflow, 0 otherwise
        if sum < value_x {
            state.registers.write(Register::VF, 1);
        } else {
            state.registers.write(Register::VF, 0);
        }
        Ok(())
    }
}

struct SubtractYFromX(DecodedInstruction);
impl Instruction for SubtractYFromX {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);
        if value_x > value_y {
            state.registers.write(reg_x, value_x - value_y);
            state.registers.write(Register::VF, 1); // No borrow
        } else {
            state.registers.write(reg_x, value_x.wrapping_sub(value_y));
            state.registers.write(Register::VF, 0); // Borrow occurred
        }
        Ok(())
    }
}

struct SubtractXFromY(DecodedInstruction);
impl Instruction for SubtractXFromY {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);
        if value_y > value_x {
            state.registers.write(reg_y, value_y - value_x);
            state.registers.write(Register::VF, 1); // No borrow
        } else {
            state.registers.write(reg_y, value_y.wrapping_sub(value_x));
            state.registers.write(Register::VF, 0); // Borrow occurred
        }
        Ok(())
    }
}

struct RightShift(DecodedInstruction);
impl Instruction for RightShift {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        // TODO: Make this configurable.
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_y = state.registers.read(reg_y);

        state.registers.write(reg_x, value_y >> 1);
        state.registers.write(Register::VF, value_y & 0x01); // Set VF to LSB before shift
        Ok(())
    }
}

struct LeftShift(DecodedInstruction);
impl Instruction for LeftShift {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        // TODO: Make this configurable.
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_y = state.registers.read(reg_y);

        state.registers.write(reg_x, value_y << 1);
        state.registers.write(Register::VF, (value_y & 0x80) >> 7); // Set VF to MSB before shift
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

struct JumpWithOffset(DecodedInstruction);
impl Instruction for JumpWithOffset {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        state.pc = usize::from(state.registers.read(Register::V0)) + self.0.nnn;
        // TODO: Make this configurable. The code below is how the CHIP-48 and SUPER-CHIP behave.
        //let reg_x = Register::from_index(self.0.x)?;
        //state.pc = (state.registers.read(reg_x) as usize + self.0.nnn);
        Ok(())
    }
}

struct Random(DecodedInstruction);
impl Instruction for Random {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let random_value = rand::random::<u8>() & self.0.nn;
        state.registers.write(reg_x, random_value);
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

struct SetVxFromTimer(DecodedInstruction);
impl Instruction for SetVxFromTimer {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        state.registers.write(reg_x, state.delay_timer);
        Ok(())
    }
}

struct SetDelayTimer(DecodedInstruction);
impl Instruction for SetDelayTimer {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        state.delay_timer = state.registers.read(reg_x);
        Ok(())
    }
}

struct SetSoundTimer(DecodedInstruction);
impl Instruction for SetSoundTimer {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        state.sound_timer = state.registers.read(reg_x);
        Ok(())
    }
}

struct AddToIndex(DecodedInstruction);
impl Instruction for AddToIndex {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let value_x = state.registers.read(reg_x);
        state.index = state.index.wrapping_add(usize::from(value_x));
        Ok(())
    }
}

struct FontChar(DecodedInstruction);
impl Instruction for FontChar {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let value_x = state.registers.read(reg_x);
        state.index = usize::from(value_x & 0x0F) * FONT_HEIGHT + FONT_ADDR;
        Ok(())
    }
}

struct BinaryCodedDecimal(DecodedInstruction);
impl Instruction for BinaryCodedDecimal {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let value_x = state.registers.read(reg_x);
        let bcd = [(value_x / 100) % 10, (value_x / 10) % 10, value_x % 10];
        for (i, &digit) in bcd.iter().enumerate() {
            state.memory.write(state.index + i, digit)?;
        }
        Ok(())
    }
}

struct Store(DecodedInstruction);
impl Instruction for Store {
    // TODO: Make this configurable so that in the alternative mode it updates index.
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        let index_rng = state.index..=state.index + self.0.x;
        let reg_rng = 0..=self.0.x;

        for (i, j) in index_rng.zip(reg_rng) {
            let reg = Register::from_index(j)?;
            let value = state.registers.read(reg);
            state.memory.write(i, value)?;
        }
        Ok(())
    }
}

struct Load(DecodedInstruction);
impl Instruction for Load {
    fn execute(&self, state: &mut ChipState) -> anyhow::Result<()> {
        // TODO: Make this configurable so that in the alternative mode it updates index.
        for i in 0..=self.0.x {
            let reg = Register::from_index(i)?;
            let value = state.registers.read(reg);
            state.memory.write(state.index + i, value)?;
        }
        Ok(())
    }
}
