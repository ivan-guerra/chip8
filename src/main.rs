//! CHIP-8 Emulator
//!
//! A complete CHIP-8 virtual machine implementation with terminal-based display,
//! audio output, and configurable execution parameters.
//!
//! # Features
//!
//! - **Complete Instruction Set**: All 35 CHIP-8 instructions implemented
//! - **Terminal Display**: 64×32 pixel game screen rendered in terminal
//! - **Audio Support**: Authentic beep sound using 440 Hz sine wave
//! - **Configurable Timing**: Adjustable frame rate and instruction speed
//! - **Keyboard Input**: Standard QWERTY to CHIP-8 keypad mapping
//! - **ROM Loading**: Support for standard CHIP-8 ROM files
//!
//! # Usage
//!
//! Run the emulator with a ROM file:
//!
//! ```bash
//! chip8 --rom-path rom/tests/games/2-ibm-logo.ch8
//! ```
//!
//! Optional parameters:
//! - `--frame-rate`: Display refresh rate (default: 60 Hz)
//! - `--ips`: Instructions per second (default: 700)
//!
//! # Controls
//!
//! The CHIP-8 keypad is mapped to QWERTY keys:
//!
//! ```text
//! CHIP-8 Keypad    QWERTY Keyboard
//! 1 2 3 C          1 2 3 4
//! 4 5 6 D          Q W E R
//! 7 8 9 E          A S D F
//! A 0 B F          Z X C V
//! ```
//!
//! Press **Escape** to exit the emulator.

use crate::emulator::Emulator;
use crate::state::{DEFAULT_FRAME_RATE, DEFAULT_INSTRUCTIONS_PER_SECOND, Settings};
use clap::Parser;

mod emulator;
mod instruction;
mod state;

#[doc(hidden)]
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = DEFAULT_FRAME_RATE, help = "Frame rate in frames per second")]
    frame_rate: u64,

    #[arg(short, long, default_value_t = DEFAULT_INSTRUCTIONS_PER_SECOND, help = "Instructions per second")]
    ips: u64,

    #[arg(short, long, help = "Path to the ROM file to run")]
    rom_path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let settings = Settings::new(args.frame_rate, args.ips, args.rom_path);
    let mut emulator = Emulator::new(settings)?;

    emulator.run()?;

    Ok(())
}
