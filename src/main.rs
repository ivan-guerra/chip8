pub mod chip8;
mod display;

fn main() -> anyhow::Result<()> {
    let mut emulator = chip8::Chip8Emulator::new();

    emulator.run(std::path::PathBuf::from("roms/ibm-logo.ch8"))?;

    Ok(())
}
