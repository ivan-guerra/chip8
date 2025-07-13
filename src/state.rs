//! CHIP-8 Emulator State Management
//!
//! This module contains all the core state components and data structures needed
//! for CHIP-8 emulation. It provides a complete implementation of the CHIP-8
//! virtual machine's architecture including memory management, register handling,
//! input processing, and display management.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use bitvec::{array::BitArray, BitArr};
use rdev::{listen, EventType, Key as RdevKey};

/// Timer value type for delay and sound timers.
/// Timers in CHIP-8 count down at 60 Hz from their initial value to zero.
pub type Timer = u8;

/// Memory address type for the CHIP-8 system.
/// Addresses range from 0x000 to 0xFFF (4096 bytes total).
pub type Address = usize;

/// Call stack for storing return addresses during subroutine calls.
/// CHIP-8 supports nested subroutines with a stack-based return mechanism.
pub type CallStack = Vec<Address>;

/// Total memory size of the CHIP-8 system in bytes.
/// The original CHIP-8 system had 4KB of RAM.
pub const MEM_SIZE: usize = 4096;

/// Starting address where the built-in font set is stored in memory.
/// Font data occupies addresses 0x50-0x9F (80 bytes for 16 characters).
pub const FONT_ADDR: Address = 0x50;

/// Height of each font character in pixels.
/// Each CHIP-8 font character is 4 pixels wide and 5 pixels tall.
pub const FONT_HEIGHT: usize = 5;

/// Default starting address for program execution.
pub const PC_START_ADDR: Address = 0x200;

/// Number of general-purpose registers in the CHIP-8 system.
/// Registers are named V0 through VF, where VF is often used as a flag register.
pub const NUM_REGISTERS: usize = 16;

/// Width of the CHIP-8 display in pixels.
/// The original CHIP-8 display was 64 pixels wide.
pub const DISPLAY_WIDTH: usize = 64;

/// Height of the CHIP-8 display in pixels.
pub const DISPLAY_HEIGHT: usize = 32;

/// Default frame rate for the emulator in frames per second.
/// This controls how often the display is refreshed and timers are decremented.
pub const DEFAULT_FRAME_RATE: u64 = 60;

/// Default instruction execution rate in instructions per second.
/// This determines how fast the CHIP-8 programs run.
pub const DEFAULT_INSTRUCTIONS_PER_SECOND: u64 = 700;

/// Memory subsystem for the CHIP-8 emulator.
///
/// Manages the 4KB memory space of the CHIP-8 system, including:
/// - Built-in font data loaded at startup
/// - ROM/program data loaded at runtime
/// - General-purpose memory for program use
///
/// Memory layout:
/// - 0x000-0x1FF: Reserved for interpreter (not used in this implementation)
/// - 0x050-0x09F: Built-in font set (16 characters, 5 bytes each)
/// - 0x200-0xFFF: Program ROM and RAM
pub struct Memory {
    data: [u8; MEM_SIZE],
}

impl Memory {
    /// Creates a new Memory instance with built-in font data pre-loaded.
    ///
    /// The font data contains hexadecimal digit sprites (0-F) that are
    /// commonly used by CHIP-8 programs for displaying numbers and letters.
    /// Each character is 4 pixels wide and 5 pixels tall.
    pub fn new() -> Self {
        let font_data = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        let data = {
            let mut data = [0; MEM_SIZE];
            data[FONT_ADDR..FONT_ADDR + font_data.len()].copy_from_slice(&font_data);
            data
        };

        Memory { data }
    }

    /// Reads a single byte from memory at the specified address.
    pub fn read(&self, addr: Address) -> anyhow::Result<u8> {
        if addr >= MEM_SIZE {
            return Err(anyhow!("Memory read out of bounds: {}", addr));
        }
        Ok(self.data[addr])
    }

    /// Writes a single byte to memory at the specified address.
    pub fn write(&mut self, addr: Address, value: u8) -> anyhow::Result<()> {
        if addr >= MEM_SIZE {
            return Err(anyhow!("Memory write out of bounds: {}", addr));
        }
        self.data[addr] = value;
        Ok(())
    }

    /// Loads a ROM file into memory starting at the program counter start address.
    ///
    /// ROM data is loaded starting at address 0x200, which is the traditional
    /// program start location for CHIP-8 systems.
    pub fn load_rom(&mut self, rom: &[u8]) -> anyhow::Result<()> {
        if rom.len() > MEM_SIZE - PC_START_ADDR {
            return Err(anyhow!("ROM too large to fit in memory"));
        }
        self.data[PC_START_ADDR..PC_START_ADDR + rom.len()].copy_from_slice(rom);
        Ok(())
    }

    /// Reads sprite data from memory for display operations.
    ///
    /// Sprites in CHIP-8 are variable-height (1-15 rows) and fixed-width (8 pixels).
    /// Each row is represented by a single byte where each bit corresponds to a pixel.
    ///
    /// # Arguments
    /// * `index` - Starting memory address of the sprite data
    /// * `rows` - Number of rows (bytes) to read for the sprite (1-15)
    ///
    /// # Returns
    /// * `Ok(&[u8])` - Slice containing the sprite data
    /// * `Err` - If the sprite data extends beyond memory bounds
    pub fn read_sprite(&self, index: Address, rows: u8) -> anyhow::Result<&[u8]> {
        let sprite_slice = index..index + rows as usize;

        if sprite_slice.end > MEM_SIZE {
            return Err(anyhow!("Sprite data out of bounds"));
        }
        Ok(&self.data[sprite_slice])
    }
}

/// Enumeration of all 16 general-purpose registers in the CHIP-8 system.
///
/// CHIP-8 has 16 8-bit registers named V0 through VF. Register VF is commonly
/// used as a flag register by arithmetic and logical operations to indicate
/// carry, borrow, or collision conditions.
#[derive(Copy, Clone)]
pub enum Register {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
}

impl Register {
    /// Converts a numeric index (0-15) to the corresponding Register enum variant.
    pub fn from_index(value: usize) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Register::V0),
            1 => Ok(Register::V1),
            2 => Ok(Register::V2),
            3 => Ok(Register::V3),
            4 => Ok(Register::V4),
            5 => Ok(Register::V5),
            6 => Ok(Register::V6),
            7 => Ok(Register::V7),
            8 => Ok(Register::V8),
            9 => Ok(Register::V9),
            10 => Ok(Register::VA),
            11 => Ok(Register::VB),
            12 => Ok(Register::VC),
            13 => Ok(Register::VD),
            14 => Ok(Register::VE),
            15 => Ok(Register::VF),
            _ => Err(anyhow!("Invalid register index: {}", value)),
        }
    }
}

/// Register bank containing all 16 general-purpose registers.
///
/// Provides a centralized interface for reading from and writing to
/// the CHIP-8's register set. All registers are 8-bit and initialized to zero.
pub struct RegisterBank {
    registers: [u8; NUM_REGISTERS],
}

impl RegisterBank {
    /// Creates a new RegisterBank with all registers initialized to zero.
    pub fn new() -> Self {
        RegisterBank {
            registers: [0; NUM_REGISTERS],
        }
    }

    /// Reads the current value of the specified register.
    pub fn read(&self, reg: Register) -> u8 {
        self.registers[reg as usize]
    }

    /// Writes a value to the specified register.
    pub fn write(&mut self, reg: Register, value: u8) {
        self.registers[reg as usize] = value;
    }
}

/// Enumeration of all 16 keys in the CHIP-8 hexadecimal keypad.
#[derive(PartialEq, Eq, Hash)]
pub enum Key {
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
}
impl Key {
    /// Converts a numeric index (0-15) to the corresponding Key enum variant.
    pub fn from_index(index: u8) -> anyhow::Result<Key> {
        match index {
            0 => Ok(Key::Key0),
            1 => Ok(Key::Key1),
            2 => Ok(Key::Key2),
            3 => Ok(Key::Key3),
            4 => Ok(Key::Key4),
            5 => Ok(Key::Key5),
            6 => Ok(Key::Key6),
            7 => Ok(Key::Key7),
            8 => Ok(Key::Key8),
            9 => Ok(Key::Key9),
            10 => Ok(Key::KeyA),
            11 => Ok(Key::KeyB),
            12 => Ok(Key::KeyC),
            13 => Ok(Key::KeyD),
            14 => Ok(Key::KeyE),
            15 => Ok(Key::KeyF),
            _ => Err(anyhow!("Invalid key index: {}", index)),
        }
    }

    /// Converts an rdev keyboard key to the corresponding CHIP-8 key.
    ///
    /// This function implements the keyboard mapping from modern QWERTY layout
    /// to the CHIP-8 hexadecimal keypad. The mapping follows a common convention
    /// used by many CHIP-8 emulators for ergonomic key placement.
    ///
    /// # Keyboard Mapping
    /// | Keyboard | CHIP-8 |
    /// |----------|--------|
    /// | 1        | 1      |
    /// | 2        | 2      |
    /// | 3        | 3      |
    /// | 4        | C      |
    /// | Q        | 4      |
    /// | W        | 5      |
    /// | E        | 6      |
    /// | R        | D      |
    /// | A        | 7      |
    /// | S        | 8      |
    /// | D        | 9      |
    /// | F        | E      |
    /// | Z        | A      |
    /// | X        | 0      |
    /// | C        | B      |
    /// | V        | F      |
    pub fn from_rdev(key: rdev::Key) -> Option<Key> {
        match key {
            RdevKey::Num1 => Some(Key::Key1),
            RdevKey::Num2 => Some(Key::Key2),
            RdevKey::Num3 => Some(Key::Key3),
            RdevKey::Num4 => Some(Key::KeyC),
            RdevKey::KeyQ => Some(Key::Key4),
            RdevKey::KeyW => Some(Key::Key5),
            RdevKey::KeyE => Some(Key::Key6),
            RdevKey::KeyR => Some(Key::KeyD),
            RdevKey::KeyA => Some(Key::Key7),
            RdevKey::KeyS => Some(Key::Key8),
            RdevKey::KeyD => Some(Key::Key9),
            RdevKey::KeyF => Some(Key::KeyE),
            RdevKey::KeyZ => Some(Key::KeyA),
            RdevKey::KeyX => Some(Key::Key0),
            RdevKey::KeyC => Some(Key::KeyB),
            RdevKey::KeyV => Some(Key::KeyF),
            _ => None,
        }
    }
}

/// Input handling system for the CHIP-8 hexadecimal keypad.
///
/// The `Keypad` struct manages key input state for the 16-key CHIP-8 keypad using
/// a background thread that listens for global keyboard events. This approach allows
/// the emulator to capture input even when not in focus and provides real-time
/// key state tracking.
///
/// # Architecture
///
/// The keypad uses a multi-threaded design:
/// - A background thread continuously listens for keyboard events using `rdev`
/// - Key states are stored in thread-safe collections (`Arc<Mutex<_>>`)
/// - The main emulator thread can query key states without blocking
///
/// # Key Mapping
///
/// Physical keyboard keys are mapped to CHIP-8 keys using the standard layout:
/// ```text
/// Keyboard    CHIP-8
/// 1 2 3 4  →  1 2 3 C
/// Q W E R  →  4 5 6 D  
/// A S D F  →  7 8 9 E
/// Z X C V  →  A 0 B F
/// ```
///
/// The Escape key is handled separately for emulator control.
///
/// # Thread Safety
///
/// All public methods are thread-safe and can be called from multiple threads
/// without external synchronization. Internal state is protected by mutexes.
pub struct Keypad {
    /// Thread-safe storage for currently pressed CHIP-8 keys.
    /// Updated by the background listener thread and read by the main emulator thread.
    pressed_keys: Arc<Mutex<HashSet<Key>>>,

    /// Thread-safe flag indicating if the Escape key is currently pressed.
    /// Used for emulator control (typically to exit the program).
    escape_pressed: Arc<Mutex<bool>>,
}

impl Keypad {
    /// Creates a new `Keypad` instance and starts the background key listener.
    ///
    /// This constructor immediately spawns a background thread that will listen
    /// for global keyboard events throughout the lifetime of the `Keypad` instance.
    /// The thread will continue running until the program terminates.
    pub fn new() -> Self {
        let pressed_keys = Arc::new(Mutex::new(HashSet::new()));
        let escape_pressed = Arc::new(Mutex::new(false));
        let pressed_keys_clone = pressed_keys.clone();
        let escape_pressed_clone = escape_pressed.clone();

        // Spawn a background thread to listen for key events
        std::thread::spawn(move || {
            if let Err(error) = listen(move |event| {
                let mut keys = pressed_keys_clone.lock().unwrap();
                let mut escape = escape_pressed_clone.lock().unwrap();

                match event.event_type {
                    EventType::KeyPress(key) => {
                        if key == RdevKey::Escape {
                            *escape = true;
                        } else if let Some(chip8_key) = Key::from_rdev(key) {
                            keys.insert(chip8_key);
                        }
                    }
                    EventType::KeyRelease(key) => {
                        if key == RdevKey::Escape {
                            *escape = false;
                        } else if let Some(chip8_key) = Key::from_rdev(key) {
                            keys.remove(&chip8_key);
                        }
                    }
                    _ => {}
                }
            }) {
                eprintln!("Error listening for key events: {:?}", error);
            }
        });

        Keypad {
            pressed_keys,
            escape_pressed,
        }
    }

    /// Checks if a specific CHIP-8 key is currently pressed (non-blocking).
    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.lock().unwrap().contains(&key)
    }

    /// Manually releases a specific CHIP-8 key from the pressed state.
    ///
    /// This method allows the emulator to programmatically release a key,
    /// which is useful for implementing certain CHIP-8 instructions that
    /// expect keys to be "consumed" after being read.
    pub fn release_key(&self, key: Key) {
        let mut keys = self.pressed_keys.lock().unwrap();

        keys.remove(&key);
    }

    /// Checks if the Escape key is currently pressed.
    pub fn is_escape_pressed(&self) -> bool {
        *self.escape_pressed.lock().unwrap()
    }
}

/// Configuration settings for the CHIP-8 emulator.
///
/// This structure holds all the runtime configuration parameters that control
/// the emulator's behavior, including timing settings and the ROM file to execute.
/// These settings are typically provided via command-line arguments and remain
/// constant throughout the emulator's execution.
pub struct Settings {
    /// Frame rate in frames per second for display updates and timer decrements.
    ///
    /// Controls how frequently the display is redrawn and the delay/sound timers
    /// are decremented. The original CHIP-8 systems typically ran at 60 Hz.
    /// Higher values make the emulator run faster, lower values make it slower.
    pub frame_rate: u64,

    /// Instruction execution rate in instructions per second.
    ///
    /// Determines how many CHIP-8 instructions are executed per second.
    /// This affects the overall speed of program execution. Original CHIP-8
    /// systems varied widely in clock speed, so this is adjustable to match
    /// different program requirements or user preferences.
    pub ips: u64,

    /// Path to the ROM file containing the CHIP-8 program to execute.
    ///
    /// This should point to a valid CHIP-8 ROM file (typically .ch8 extension).
    pub rom: PathBuf,
}

impl Settings {
    /// Creates a new Settings instance with the specified parameters.
    pub fn new(frame_rate: u64, ips: u64, rom: String) -> Self {
        Settings {
            frame_rate,
            ips,
            rom: rom.into(),
        }
    }
}

/// Core state container for the CHIP-8 emulator.
///
/// This structure holds all the runtime state necessary for CHIP-8 emulation,
/// including CPU registers, memory, display buffer, input handling, and timers.
/// It serves as the central state object that is passed between instruction
/// executions and system components.
pub struct Chip8State {
    /// Emulator configuration settings (frame rate, instruction speed, ROM path).
    pub settings: Settings,

    /// 4KB memory subsystem containing font data, ROM, and runtime memory.
    pub memory: Memory,

    /// Bank of 16 general-purpose 8-bit registers (V0-VF).
    pub registers: RegisterBank,

    /// Program counter pointing to the current instruction address.
    pub pc: Address,

    /// Index register used for memory addressing in certain instructions.
    pub index: Address,

    /// Call stack for subroutine return addresses.
    pub stack: CallStack,

    /// Delay timer that counts down at 60Hz, used for timing game events.
    pub delay_timer: Timer,

    /// Sound timer that counts down at 60Hz, triggers audio when non-zero.
    pub sound_timer: Timer,

    /// Display buffer representing the 64×32 monochrome screen.
    /// Each bit represents one pixel (true = on, false = off).
    pub display: BitArr!(for DISPLAY_WIDTH * DISPLAY_HEIGHT),

    /// Input handling system for the 16-key hexadecimal keypad.
    pub keypad: Keypad,
}

impl Chip8State {
    /// Creates a new CHIP-8 system state with default initialization.
    pub fn new(settings: Settings) -> Self {
        Chip8State {
            settings,
            memory: Memory::new(),
            registers: RegisterBank::new(),
            pc: PC_START_ADDR,
            index: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            display: BitArray::ZERO,
            keypad: Keypad::new(),
        }
    }

    /// Clears all pixels on the display screen.
    pub fn clear_display(&mut self) {
        self.display.fill(false);
    }

    /// Draws a sprite on the display and detects pixel collisions.
    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite_idx: u8) -> anyhow::Result<bool> {
        let mut collision = false;
        let sprite = self.memory.read_sprite(self.index, sprite_idx)?;

        for (row, &byte) in sprite.iter().enumerate() {
            for bit in 0..8 {
                let pixel_x = x + bit;
                let pixel_y = y + row;

                // Skip pixels that are outside screen boundaries
                if pixel_x >= DISPLAY_WIDTH || pixel_y >= DISPLAY_HEIGHT {
                    continue;
                }

                let index = pixel_y * DISPLAY_WIDTH + pixel_x;
                let current_pixel = self.display[index];
                let new_pixel = (byte >> (7 - bit)) & 1 == 1;

                collision = current_pixel && new_pixel;

                self.display.set(index, current_pixel ^ new_pixel);
            }
        }
        Ok(collision)
    }
}
