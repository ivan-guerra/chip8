use crate::chip8::Emulator;
use crate::state::{Settings, DEFAULT_FRAME_RATE};
use clap::Parser;

mod chip8;
mod instruction;
mod state;

#[doc(hidden)]
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = DEFAULT_FRAME_RATE, help = "Frame rate in frames per second")]
    frame_rate: u64,

    #[arg(short, long, help = "Path to the ROM file to run")]
    rom_path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let settings = Settings::new(args.frame_rate, args.rom_path);
    let mut emulator = Emulator::new(settings);

    emulator.run()?;

    Ok(())
}
