pub mod chip8;
mod components;
mod display;
mod instruction;

fn main() -> anyhow::Result<()> {
    let mut emulator = chip8::Chip8Emulator::default();

    emulator.run(std::path::PathBuf::from("roms/ibm-logo.ch8"))?;

    Ok(())
}
