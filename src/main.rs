use crate::chip8::Emulator;
use crate::state::{DEFAULT_FRAME_RATE, Settings};
use clap::Parser;
use state::DEFAULT_INSTRUCTIONS_PER_SECOND;

mod chip8;
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
    let mut emulator = Emulator::new(settings);

    emulator.run()?;

    Ok(())
}
