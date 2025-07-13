//! CHIP-8 Instruction Set Implementation
//!
//! This module provides a complete implementation of the CHIP-8 instruction set,
//! including instruction decoding and execution. The CHIP-8 virtual machine has
//! 35 different instructions that cover arithmetic, logic, memory operations,
//! control flow, graphics, and input handling.

use anyhow::anyhow;

use crate::state::{
    Address, Chip8State, Key, Register, DISPLAY_HEIGHT, DISPLAY_WIDTH, FONT_ADDR, FONT_HEIGHT,
};

/// Trait defining the execution interface for CHIP-8 instructions.
///
/// All CHIP-8 instructions implement this trait to provide a uniform interface
/// for instruction execution. Instructions operate on mutable state and can
/// return errors if execution fails (e.g., invalid memory access, stack overflow).
pub trait Instruction {
    /// Executes the instruction, possibly modifying the provided CHIP-8 state.
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()>;
}

/// Decodes a raw 16-bit instruction word into an executable instruction object.
///
/// This function implements the complete CHIP-8 instruction decoder, parsing
/// the opcode and creating the appropriate instruction implementation. The CHIP-8
/// instruction set uses a variable-length opcode format where the first nibble
/// determines the instruction family, and additional nibbles may further specify
/// the exact operation.
///
/// # Instruction Format
/// CHIP-8 instructions are 16-bit words with the following general structure:
/// ```text
/// OPCODE XY N    (where each character represents a 4-bit nibble)
/// OPCODE X NN    (8-bit immediate value)
/// OPCODE NNN     (12-bit address)
/// ```
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
            0x7 => Ok(Box::new(SubtractXFromY(decoded))),
            0xE => Ok(Box::new(LeftShift(decoded))),
            _ => Err(anyhow!("Unsupported opcode for 0x8: {:#04X}", raw)),
        },
        0x9 => Ok(Box::new(JumpXNeqY(decoded))),
        0xA => Ok(Box::new(SetIndex(decoded))),
        0xB => Ok(Box::new(JumpWithOffset(decoded))),
        0xC => Ok(Box::new(Random(decoded))),
        0xD => Ok(Box::new(Display(decoded))),
        0xE => match decoded.nn {
            0x9E => Ok(Box::new(SkipIfKeyPressed(decoded))),
            0xA1 => Ok(Box::new(SkipIfKeyNotPressed(decoded))),
            _ => Err(anyhow!("Unsupported opcode for 0xE: {:#04X}", raw)),
        },
        0xF => match decoded.nn {
            0x07 => Ok(Box::new(SetVxFromTimer(decoded))),
            0x15 => Ok(Box::new(SetDelayTimer(decoded))),
            0x18 => Ok(Box::new(SetSoundTimer(decoded))),
            0x1E => Ok(Box::new(AddToIndex(decoded))),
            0x29 => Ok(Box::new(FontChar(decoded))),
            0x33 => Ok(Box::new(BinaryCodedDecimal(decoded))),
            0x55 => Ok(Box::new(Store(decoded))),
            0x65 => Ok(Box::new(Load(decoded))),
            0x0A => Ok(Box::new(GetKey(decoded))),
            _ => Err(anyhow!("Unsupported opcode for 0xF: {:#04X}", raw)),
        },
        _ => Err(anyhow!("Unknown opcode: {:#04X}", decoded.opcode)),
    }
}

/// Internal structure for holding parsed components of a CHIP-8 instruction.
///
/// This structure breaks down a 16-bit instruction word into its constituent
/// parts for easier access during instruction execution.
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
    /// Creates a new DecodedInstruction by parsing a raw 16-bit instruction word.
    ///
    /// This function extracts all possible instruction components using bit
    /// manipulation, regardless of which components the specific instruction
    /// actually uses.
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

/// Clears the entire display screen.
///
/// Implements the CHIP-8 instruction `00E0` which sets all pixels on the
/// 64×32 display to off (black).
struct ClearScreen;
impl Instruction for ClearScreen {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        state.clear_display();
        Ok(())
    }
}

/// Unconditional jump to a specific memory address.
///
/// Implements the CHIP-8 instruction `1NNN` which sets the program counter
/// to the address NNN. This causes execution to continue from that location.
struct Jump(DecodedInstruction);
impl Instruction for Jump {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        state.pc = self.0.nnn;
        Ok(())
    }
}

/// Calls a subroutine at the specified address.
///
/// Implements the CHIP-8 instruction `2NNN` which:
/// 1. Pushes the current program counter onto the call stack
/// 2. Sets the program counter to address NNN
///
/// This allows for nested subroutine calls with proper return handling.
/// The call stack stores return addresses for when the subroutine returns.
struct SubroutineCall(DecodedInstruction);
impl Instruction for SubroutineCall {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        state.stack.push(state.pc);
        state.pc = self.0.nnn;
        Ok(())
    }
}

/// Returns from a subroutine call.
///
/// Implements the CHIP-8 instruction `00EE` which:
/// 1. Pops the top address from the call stack
/// 2. Sets the program counter to that address
///
/// This returns execution to the instruction immediately following
/// the most recent subroutine call. If the stack is empty, an error
/// is returned indicating stack underflow.
struct SubroutineReturn;
impl Instruction for SubroutineReturn {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        if let Some(return_address) = state.stack.pop() {
            state.pc = return_address;
            Ok(())
        } else {
            Err(anyhow!("Stack underflow: No return address available"))
        }
    }
}

/// Conditional skip if register Vx equals immediate value.
///
/// Implements the CHIP-8 instruction `3XNN` which compares the value
/// in register Vx with the immediate value NN. If they are equal,
/// the next instruction is skipped by advancing the program counter
/// by 2 additional bytes.
struct JumpEqX(DecodedInstruction);
impl Instruction for JumpEqX {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        if state.registers.read(reg_x) == self.0.nn {
            state.pc += 2;
        }
        Ok(())
    }
}

/// Conditional skip if register Vx does not equal immediate value.
///
/// Implements the CHIP-8 instruction `4XNN` which compares the value
/// in register Vx with the immediate value NN. If they are not equal,
/// the next instruction is skipped by advancing the program counter
/// by 2 additional bytes.
struct JumpNeqX(DecodedInstruction);
impl Instruction for JumpNeqX {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        if state.registers.read(reg_x) != self.0.nn {
            state.pc += 2;
        }
        Ok(())
    }
}

/// Conditional skip if register Vx equals register Vy.
///
/// Implements the CHIP-8 instruction `5XY0` which compares the values
/// in registers Vx and Vy. If they are equal, the next instruction
/// is skipped by advancing the program counter by 2 additional bytes.
struct JumpXEqY(DecodedInstruction);
impl Instruction for JumpXEqY {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        if state.registers.read(reg_x) == state.registers.read(reg_y) {
            state.pc += 2;
        }
        Ok(())
    }
}

/// Conditional skip if register Vx does not equal register Vy.
///
/// Implements the CHIP-8 instruction `9XY0` which compares the values
/// in registers Vx and Vy. If they are not equal, the next instruction
/// is skipped by advancing the program counter by 2 additional bytes.
struct JumpXNeqY(DecodedInstruction);
impl Instruction for JumpXNeqY {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        if state.registers.read(reg_x) != state.registers.read(reg_y) {
            state.pc += 2;
        }
        Ok(())
    }
}

/// Sets register Vx to an immediate 8-bit value.
///
/// Implements the CHIP-8 instruction `6XNN` which loads the immediate
/// value NN into register Vx. This is the primary way to initialize
/// registers with constant values.
struct SetImmediate(DecodedInstruction);
impl Instruction for SetImmediate {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        state.registers.write(reg_x, self.0.nn);
        Ok(())
    }
}

/// Adds an immediate 8-bit value to register Vx.
///
/// Implements the CHIP-8 instruction `7XNN` which adds the immediate
/// value NN to the current value in register Vx. The addition uses
/// wrapping arithmetic (overflow wraps around to 0). Unlike `BinaryAdd`,
/// this instruction does not set the carry flag (VF).
struct Add(DecodedInstruction);
impl Instruction for Add {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_x_val = state.registers.read(reg_x);
        state
            .registers
            .write(reg_x, reg_x_val.wrapping_add(self.0.nn));
        Ok(())
    }
}

/// Copies the value from register Vy to register Vx.
///
/// Implements the CHIP-8 instruction `8XY0` which sets register Vx
/// to the same value as register Vy. The value in Vy remains unchanged.
struct SetXToY(DecodedInstruction);
impl Instruction for SetXToY {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_y = state.registers.read(reg_y);
        state.registers.write(reg_x, value_y);
        Ok(())
    }
}

/// Performs bitwise OR operation between registers Vx and Vy.
///
/// Implements the CHIP-8 instruction `8XY1` which performs a bitwise OR
/// operation on the values in registers Vx and Vy, storing the result
/// in Vx. As a side effect, register VF is always set to 0.
struct BinaryOr(DecodedInstruction);
impl Instruction for BinaryOr {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);

        state.registers.write(reg_x, value_x | value_y);
        state.registers.write(Register::VF, 0);
        Ok(())
    }
}

/// Performs bitwise AND operation between registers Vx and Vy.
///
/// Implements the CHIP-8 instruction `8XY2` which performs a bitwise AND
/// operation on the values in registers Vx and Vy, storing the result
/// in Vx. As a side effect, register VF is always set to 0.
struct BinaryAnd(DecodedInstruction);
impl Instruction for BinaryAnd {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);

        state.registers.write(reg_x, value_x & value_y);
        state.registers.write(Register::VF, 0);
        Ok(())
    }
}

/// Performs bitwise XOR operation between registers Vx and Vy.
///
/// Implements the CHIP-8 instruction `8XY3` which performs a bitwise XOR
/// (exclusive OR) operation on the values in registers Vx and Vy, storing
/// the result in Vx. As a side effect, register VF is always set to 0.
struct LogicalXor(DecodedInstruction);
impl Instruction for LogicalXor {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);

        state.registers.write(reg_x, value_x ^ value_y);
        state.registers.write(Register::VF, 0);
        Ok(())
    }
}

/// Adds register Vy to register Vx with carry detection.
///
/// Implements the CHIP-8 instruction `8XY4` which adds the value in register
/// Vy to register Vx, storing the result in Vx. If the addition results in
/// a value greater than 255 (8-bit overflow), register VF is set to 1 to
/// indicate carry; otherwise VF is set to 0. The actual addition uses
/// wrapping arithmetic.
struct BinaryAdd(DecodedInstruction);
impl Instruction for BinaryAdd {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);
        let sum = value_x.wrapping_add(value_y);
        state.registers.write(reg_x, sum);

        if sum < value_x {
            state.registers.write(Register::VF, 1); // No borrow
        } else {
            state.registers.write(Register::VF, 0); // Borrow occurred
        }
        Ok(())
    }
}

/// Subtracts register Vy from register Vx with borrow detection.
///
/// Implements the CHIP-8 instruction `8XY5` which subtracts the value in register
/// Vy from register Vx, storing the result in Vx. If Vx >= Vy (no borrow needed),
/// register VF is set to 1; otherwise VF is set to 0 to indicate a borrow occurred.
/// The actual subtraction uses wrapping arithmetic when borrow occurs.
struct SubtractYFromX(DecodedInstruction);
impl Instruction for SubtractYFromX {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);

        if value_x >= value_y {
            state.registers.write(reg_x, value_x - value_y);
            state.registers.write(Register::VF, 1); // No borrow
        } else {
            state.registers.write(reg_x, value_x.wrapping_sub(value_y));
            state.registers.write(Register::VF, 0); // Borrow occurred
        }
        Ok(())
    }
}

/// Subtracts register Vx from register Vy with borrow detection.
///
/// Implements the CHIP-8 instruction `8XY7` which subtracts the value in register
/// Vx from register Vy, storing the result in Vx. If Vy >= Vx (no borrow needed),
/// register VF is set to 1; otherwise VF is set to 0 to indicate a borrow occurred.
/// The actual subtraction uses wrapping arithmetic when borrow occurs.
struct SubtractXFromY(DecodedInstruction);
impl Instruction for SubtractXFromY {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_x = state.registers.read(reg_x);
        let value_y = state.registers.read(reg_y);

        if value_y >= value_x {
            state.registers.write(reg_x, value_y - value_x);
            state.registers.write(Register::VF, 1); // No borrow
        } else {
            state.registers.write(reg_x, value_y.wrapping_sub(value_x));
            state.registers.write(Register::VF, 0); // Borrow occurred
        }
        Ok(())
    }
}

/// Performs a right shift operation on register Vy, storing result in Vx.
///
/// Implements the CHIP-8 instruction `8XY6` which shifts the value in register
/// Vy one bit to the right and stores the result in register Vx. The least
/// significant bit (LSB) of the original value is stored in register VF before
/// the shift operation.
struct RightShift(DecodedInstruction);
impl Instruction for RightShift {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_y = state.registers.read(reg_y);
        let value_x = value_y;

        state.registers.write(reg_x, value_x >> 1);
        state.registers.write(Register::VF, value_x & 0x01); // Set VF to LSB before shift
        Ok(())
    }
}

/// Performs a left shift operation on register Vy, storing result in Vx.
///
/// Implements the CHIP-8 instruction `8XYE` which shifts the value in register
/// Vy one bit to the left and stores the result in register Vx. The most
/// significant bit (MSB) of the original value is stored in register VF before
/// the shift operation.
struct LeftShift(DecodedInstruction);
impl Instruction for LeftShift {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let reg_y = Register::from_index(self.0.y)?;
        let value_y = state.registers.read(reg_y);
        let value_x = value_y;

        state.registers.write(reg_x, value_x << 1);
        state.registers.write(Register::VF, (value_x & 0x80) >> 7); // Set VF to MSB before shift
        Ok(())
    }
}

/// Sets the index register to a specific memory address.
///
/// Implements the CHIP-8 instruction `ANNN` which loads the immediate
/// value NNN into the index register. The index register is used by
/// various instructions for memory addressing operations.
struct SetIndex(DecodedInstruction);
impl Instruction for SetIndex {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        state.index = self.0.nnn;
        Ok(())
    }
}

/// Jumps to address NNN plus the value in register V0.
///
/// Implements the CHIP-8 instruction `BNNN` which sets the program counter
/// to the address NNN plus the value stored in register V0. This allows
/// for computed jumps based on runtime values.
struct JumpWithOffset(DecodedInstruction);
impl Instruction for JumpWithOffset {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        state.pc = usize::from(state.registers.read(Register::V0)) + self.0.nnn;
        Ok(())
    }
}

/// Generates a random number and applies a bitmask.
///
/// Implements the CHIP-8 instruction `CXNN` which generates a random 8-bit
/// number, performs a bitwise AND operation with the immediate value NN,
/// and stores the result in register Vx.
struct Random(DecodedInstruction);
impl Instruction for Random {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let random_value = rand::random::<u8>() & self.0.nn;
        state.registers.write(reg_x, random_value);
        Ok(())
    }
}

/// Draws a sprite to the display with collision detection.
///
/// Implements the CHIP-8 instruction `DXYN` which draws an N-byte sprite
/// starting at memory location I at coordinates (Vx, Vy) on the display.
/// Each sprite is 8 pixels wide and N pixels tall. Pixels are XORed with
/// existing pixels, and VF is set to 1 if any pixels are erased (collision).
///
/// # Collision Detection
/// If any existing pixel is turned off by the XOR operation, VF is set to 1.
/// This is used by games to detect when sprites overlap.
struct Display(DecodedInstruction);
impl Instruction for Display {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let x = state.registers.read(Register::from_index(self.0.x)?);
        let y = state.registers.read(Register::from_index(self.0.y)?);

        state.registers.write(Register::VF, 0);
        if state.draw_sprite(
            usize::from(x) % DISPLAY_WIDTH,
            usize::from(y) % DISPLAY_HEIGHT,
            self.0.n,
        )? {
            state.registers.write(Register::VF, 1);
        } else {
            state.registers.write(Register::VF, 0);
        }
        Ok(())
    }
}

/// Skips the next instruction if the specified key is pressed.
///
/// Implements the CHIP-8 instruction `EX9E` which checks if the key
/// corresponding to the value in register Vx is currently pressed.
/// If the key is pressed, the program counter is advanced by 2 bytes
/// to skip the next instruction.
struct SkipIfKeyPressed(DecodedInstruction);
impl Instruction for SkipIfKeyPressed {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let value_x = state.registers.read(reg_x);
        let pressed_key = Key::from_index(value_x)?;
        let is_key_pressed = state.keypad.is_key_pressed(pressed_key);

        if is_key_pressed {
            state.pc += 2;
        }
        Ok(())
    }
}

/// Skips the next instruction if the specified key is not pressed.
///
/// Implements the CHIP-8 instruction `EXA1` which checks if the key
/// corresponding to the value in register Vx is currently not pressed.
/// If the key is not pressed, the program counter is advanced by 2 bytes
/// to skip the next instruction.
struct SkipIfKeyNotPressed(DecodedInstruction);
impl Instruction for SkipIfKeyNotPressed {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let value_x = state.registers.read(reg_x);
        let pressed_key = Key::from_index(value_x)?;
        let is_key_pressed = state.keypad.is_key_pressed(pressed_key);

        if !is_key_pressed {
            state.pc += 2;
        }
        Ok(())
    }
}

/// Sets register Vx to the current value of the delay timer.
///
/// Implements the CHIP-8 instruction `FX07` which reads the current
/// value of the delay timer and stores it in register Vx. This allows
/// programs to check timing and synchronize events.
struct SetVxFromTimer(DecodedInstruction);
impl Instruction for SetVxFromTimer {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        state.registers.write(reg_x, state.delay_timer);
        Ok(())
    }
}

/// Sets the delay timer to the value in register Vx.
///
/// Implements the CHIP-8 instruction `FX15` which sets the delay timer
/// to the value stored in register Vx. The delay timer counts down at
/// 60 Hz until it reaches zero.
struct SetDelayTimer(DecodedInstruction);
impl Instruction for SetDelayTimer {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        state.delay_timer = state.registers.read(reg_x);
        Ok(())
    }
}

/// Sets the sound timer to the value in register Vx.
///
/// Implements the CHIP-8 instruction `FX18` which sets the sound timer
/// to the value stored in register Vx. The sound timer counts down at
/// 60 Hz, and a beep sound is played while the timer is non-zero.
struct SetSoundTimer(DecodedInstruction);
impl Instruction for SetSoundTimer {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        state.sound_timer = state.registers.read(reg_x);
        Ok(())
    }
}

/// Adds the value in register Vx to the index register.
///
/// Implements the CHIP-8 instruction `FX1E` which adds the value in
/// register Vx to the current value of the index register (I).
/// Uses wrapping arithmetic to handle overflow.
struct AddToIndex(DecodedInstruction);
impl Instruction for AddToIndex {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let value_x = state.registers.read(reg_x);
        state.index = state.index.wrapping_add(usize::from(value_x));
        Ok(())
    }
}

/// Sets the index register to the location of a hexadecimal character sprite.
///
/// Implements the CHIP-8 instruction `FX29` which sets the index register
/// to the memory location of the sprite data for the hexadecimal digit
/// stored in register Vx. Only the lower 4 bits of Vx are used (0-F).
struct FontChar(DecodedInstruction);
impl Instruction for FontChar {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let value_x = state.registers.read(reg_x);
        state.index = usize::from(value_x & 0x0F) * FONT_HEIGHT + FONT_ADDR;
        Ok(())
    }
}

/// Stores the binary-coded decimal representation of register Vx.
///
/// Implements the CHIP-8 instruction `FX33` which takes the decimal value
/// in register Vx and stores the hundreds digit at I, tens digit at I+1,
/// and ones digit at I+2. This converts a binary number to its decimal
/// representation for display purposes.
struct BinaryCodedDecimal(DecodedInstruction);
impl Instruction for BinaryCodedDecimal {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let reg_x = Register::from_index(self.0.x)?;
        let value_x = state.registers.read(reg_x);
        let bcd = [(value_x / 100) % 10, (value_x / 10) % 10, value_x % 10];
        for (i, &digit) in bcd.iter().enumerate() {
            state.memory.write(state.index + i, digit)?;
        }
        Ok(())
    }
}

/// Stores registers V0 through Vx in memory starting at the index register.
///
/// Implements the CHIP-8 instruction `FX55` which stores the values from registers
/// V0 through Vx (inclusive) into memory starting at the address stored in the
/// index register (I). After the operation, the index register is incremented
/// by x+1 to point to the next available memory location.
struct Store(DecodedInstruction);
impl Instruction for Store {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let index_rng = state.index..=state.index + self.0.x;
        let reg_rng = 0..=self.0.x;

        for (i, j) in index_rng.zip(reg_rng) {
            let reg = Register::from_index(j)?;
            let value = state.registers.read(reg);
            state.memory.write(i, value)?;
        }
        state.index = state.index.wrapping_add(self.0.x + 1);
        Ok(())
    }
}

/// Loads memory values into registers V0 through Vx from the index register location.
///
/// Implements the CHIP-8 instruction `FX65` which loads values from memory starting
/// at the address stored in the index register (I) into registers V0 through Vx
/// (inclusive). After the operation, the index register is incremented by x+1
/// to point to the next available memory location.
struct Load(DecodedInstruction);
impl Instruction for Load {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        for i in 0..=self.0.x {
            let value = state.memory.read(state.index + i)?;
            let reg = Register::from_index(i)?;
            state.registers.write(reg, value);
        }
        state.index = state.index.wrapping_add(self.0.x + 1);
        Ok(())
    }
}

/// Waits for a key press and stores the pressed key value in register Vx.
///
/// Implements the CHIP-8 instruction `FX0A` which blocks execution until any
/// key on the hexadecimal keypad is pressed. Once a key is pressed, its value
/// (0-F) is stored in register Vx and the key is automatically released to
/// ensure proper single-key semantics.
///
/// # Blocking Behavior
/// If no key is currently pressed, the program counter is decremented by 2
/// to repeat this instruction on the next cycle, effectively creating a
/// busy-wait loop until a key becomes available.
struct GetKey(DecodedInstruction);
impl Instruction for GetKey {
    fn execute(&self, state: &mut Chip8State) -> anyhow::Result<()> {
        let pressed_key = (0..=15).find(|i| {
            let key = Key::from_index(*i).unwrap();
            state.keypad.is_key_pressed(key)
        });

        if let Some(i) = pressed_key {
            let reg_x = Register::from_index(self.0.x)?;
            state.registers.write(reg_x, i);

            // Need to manually mark key as released as Chip-8 expects the key to be released after
            // reading
            state.keypad.release_key(Key::from_index(i)?);
        } else {
            state.pc -= 2; // If no key is pressed, decrement PC to repeat the instruction
        }

        Ok(())
    }
}
