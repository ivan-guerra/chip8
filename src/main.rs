pub mod chip8;
mod instruction;
mod state;

fn main() -> anyhow::Result<()> {
    let mut emulator = chip8::Emulator::default();
    emulator.run(std::path::PathBuf::from("roms/ibm-logo.ch8"))?;

    Ok(())
}
