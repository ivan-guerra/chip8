pub struct Chip8Emulator {
    memory: [u8; 4096],
    registers: [u8; 16],
    pc: usize,
    stack: Vec<usize>,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],
}

impl Default for Chip8Emulator {
    fn default() -> Self {
        Chip8Emulator {
            memory: [0; 4096],
            registers: [0; 16],
            pc: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
        }
    }
}

impl Chip8Emulator {
    pub fn new() -> Self {
        Chip8Emulator::default()
    }

    pub fn run(&mut self) {
        loop {
            // TODO: Noop for now.
        }
    }
}
