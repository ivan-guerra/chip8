pub mod chip8;
mod display;

fn main() {
    let mut emulator = chip8::Chip8Emulator::new();
    emulator.run();
}
