//! CHIP-8 Emulator Core
//!
//! This module implements the main emulator interface for the CHIP-8 virtual machine,
//! providing the complete execution environment including audio output, display rendering,
//! and the primary emulation loop.

use std::time::{Duration, Instant};

use anyhow::anyhow;
use crossterm::{
    event,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use rodio::{OutputStream, Sink, Source, source::SineWave};

use crate::instruction::{Instruction, decode};
use crate::state::{Chip8State, DISPLAY_HEIGHT, DISPLAY_WIDTH, MEM_SIZE, Settings};

/// Default frequency for the CHIP-8 beep sound in Hz.
const DEFAULT_FREQUENCY: f32 = 440.0;

/// Audio subsystem for the CHIP-8 emulator's sound timer functionality.
///
/// The `Beep` struct manages audio output for the CHIP-8's sound timer system.
/// When the sound timer is non-zero, a continuous tone is played. When the timer
/// reaches zero, the sound stops. This implements the original CHIP-8's simple
/// audio capabilities using the `rodio` audio library.
pub struct Beep {
    /// Audio sink that controls playback of the generated tone.
    /// Used to start and stop the beep sound based on the sound timer state.
    sink: Sink,

    /// Audio output stream handle.
    /// Must be kept alive for the duration of audio playback. Dropping this
    /// would terminate the audio connection and cause the sink to become invalid.
    #[allow(dead_code)]
    stream: OutputStream,
}

impl Beep {
    /// Creates a new `Beep` instance with the specified frequency.
    ///
    /// This constructor initializes the audio subsystem and prepares a continuous
    /// sine wave tone. The audio starts in a paused state and must be explicitly
    /// activated using the `on()` method.
    pub fn new(freq: f32) -> anyhow::Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        let source = SineWave::new(freq).repeat_infinite();

        sink.append(source);
        sink.pause();

        Ok(Self { sink, stream })
    }

    /// Starts playing the beep tone.
    ///
    /// Resumes playback of the sine wave tone. If the tone is already playing,
    /// this method has no effect. The tone will continue playing until `off()`
    /// is called or the `Beep` instance is dropped.
    pub fn on(&mut self) {
        self.sink.play();
    }

    /// Stops playing the beep tone.
    ///
    /// Pauses playback of the sine wave tone. If the tone is already stopped,
    /// this method has no effect. The audio system remains ready and can be
    /// restarted immediately with `on()`.
    pub fn off(&mut self) {
        self.sink.pause();
    }
}

/// Main emulator struct that encapsulates the CHIP-8 virtual machine state and
/// audio subsystem.
pub struct Emulator {
    state: Chip8State,
    beeper: Beep,
}

impl Emulator {
    /// Renders the complete emulator interface including game screen and key mapping.
    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, rom_name: &str) {
        use ratatui::layout::{Constraint, Direction, Layout};

        // Calculate the exact size needed for 64x32 display plus borders
        let game_height = (DISPLAY_HEIGHT as u16) + 2; // +2 for top and bottom borders

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(game_height), // Exact size for game area
                Constraint::Length(7),           // Key mapping area
                Constraint::Min(0),              // Remaining space
            ])
            .split(area);

        self.draw_main_screen(frame, chunks[0], rom_name);
        self.draw_key_mapping(frame, chunks[1]);
    }

    /// Renders the main CHIP-8 game screen with proper centering and borders.
    ///
    /// This function handles the display of the 64×32 pixel game area, including:
    /// - Horizontal centering when the terminal is wider than needed
    /// - Converting the bit-based display buffer to visual characters
    /// - Adding a border with the ROM name as the title
    fn draw_main_screen(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        rom_name: &str,
    ) {
        use ratatui::layout::{Constraint, Direction, Layout};

        let game_width = (DISPLAY_WIDTH as u16) + 2; // +2 for left and right borders

        // Center the game horizontally if the terminal is wider than needed
        let game_area = if area.width > game_width {
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(game_width),
                    Constraint::Min(0),
                ])
                .split(area);
            horizontal_chunks[1]
        } else {
            area
        };

        // Convert display buffer to string representation
        let mut row_string = String::with_capacity(DISPLAY_WIDTH * DISPLAY_HEIGHT + DISPLAY_HEIGHT);
        for row_idx in 0..DISPLAY_HEIGHT {
            for col_idx in 0..DISPLAY_WIDTH {
                let index = row_idx * DISPLAY_WIDTH + col_idx;
                row_string.push(if self.state.display[index] {
                    '█'
                } else {
                    ' '
                });
            }
            row_string.push('\n');
        }

        let game_paragraph = Paragraph::new(row_string)
            .block(Block::default().borders(Borders::ALL).title(rom_name))
            .style(Style::default().fg(Color::White));
        frame.render_widget(game_paragraph, game_area);
    }

    /// Renders the keyboard mapping reference panel.
    fn draw_key_mapping(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let key_mapping = "Key Mapping:\n\
    1 2 3 4    →    1 2 3 C\n\
    Q W E R    →    4 5 6 D\n\
    A S D F    →    7 8 9 E\n\
    Z X C V    →    A 0 B F";

        let key_paragraph = Paragraph::new(key_mapping)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Keypad"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(key_paragraph, area);
    }

    /// Fetches and decodes the next instruction from memory.
    ///
    /// This method reads a 16-bit instruction from the current program counter location,
    /// advances the program counter by 2 bytes, and decodes the raw instruction into
    /// an executable instruction object.
    fn fetch_instruction(&mut self) -> anyhow::Result<Box<dyn Instruction>> {
        if self.state.pc + 1 >= MEM_SIZE {
            return Err(anyhow!("Program counter out of bounds"));
        }
        let high_byte = u16::from(self.state.memory.read(self.state.pc)?);
        let low_byte = u16::from(self.state.memory.read(self.state.pc + 1)?);

        // Move the program counter to next instruction
        self.state.pc += 2;

        decode((high_byte << 8) | low_byte)
    }

    /// Creates a new emulator instance with the provided configuration settings.
    ///
    /// This constructor initializes all emulator subsystems including:
    /// - CHIP-8 system state (memory, registers, timers, display, input)
    /// - Audio subsystem with default beep frequency (440 Hz)
    pub fn new(settings: Settings) -> anyhow::Result<Self> {
        Ok(Emulator {
            state: Chip8State::new(settings),
            beeper: Beep::new(DEFAULT_FREQUENCY)?,
        })
    }

    /// Starts the main emulation loop and runs the CHIP-8 program.
    ///
    /// This method handles the complete emulation lifecycle including:
    /// - Terminal setup and ROM loading
    /// - Main execution loop with precise timing control
    /// - Input processing and display rendering
    /// - Audio management based on sound timer
    /// - Cleanup and terminal restoration
    ///
    /// # Execution Flow
    /// 1. **Initialization**: Sets up terminal UI, loads ROM into memory
    /// 2. **Main Loop**: Runs until Escape key is pressed, each iteration:
    ///    - Processes input events
    ///    - Decrements delay and sound timers (60 Hz)
    ///    - Executes calculated number of instructions per frame
    ///    - Updates display and audio output
    ///    - Maintains precise frame timing through sleep
    /// 3. **Cleanup**: Restores terminal to normal mode
    ///
    /// # Timing Model
    /// The emulator uses a frame-based timing model where:
    /// - Display refreshes at the configured frame rate (default 60 Hz)
    /// - Timers decrement once per frame
    /// - Instructions execute at `ips / frame_rate` per frame
    /// - Frame duration is maintained through sleep compensation
    ///
    /// # Input Handling
    /// - CHIP-8 keypad input is handled via global key listener
    /// - Escape key exits the emulator
    /// - Terminal events are consumed to prevent echo/interference
    ///
    /// # Audio Management
    /// - Sound timer > 0: Continuous beep tone plays
    /// - Sound timer = 0: Audio output stops
    /// - Uses 440 Hz sine wave for authentic CHIP-8 sound
    pub fn run(&mut self) -> anyhow::Result<()> {
        let frame_duration = Duration::from_secs_f64(1.0 / self.state.settings.frame_rate as f64);
        let instructions_per_frame = self.state.settings.ips / self.state.settings.frame_rate;
        let rom_stem: String = self
            .state
            .settings
            .rom
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown ROM".to_string());
        let rom_data = std::fs::read(self.state.settings.rom.clone())?;

        enable_raw_mode()?;
        let stdout = std::io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        self.state.memory.load_rom(&rom_data)?;
        'mainloop: loop {
            let frame_start = Instant::now();

            if self.state.keypad.is_escape_pressed() {
                terminal.clear()?;
                break 'mainloop;
            }

            // Consume and discard any crossterm events to prevent echoing
            while event::poll(Duration::ZERO)? {
                let _ = event::read()?;
            }

            terminal.try_draw(|frame| -> std::io::Result<()> {
                self.state.delay_timer = self.state.delay_timer.saturating_sub(1);
                self.state.sound_timer = self.state.sound_timer.saturating_sub(1);

                if self.state.sound_timer == 0 {
                    self.beeper.off();
                } else {
                    self.beeper.on();
                }

                for _ in 0..=instructions_per_frame {
                    let instruction = self
                        .fetch_instruction()
                        .map_err(|e| std::io::Error::other(e.to_string()))?;
                    instruction
                        .execute(&mut self.state)
                        .map_err(|e| std::io::Error::other(e.to_string()))?;
                }

                self.draw(frame, frame.area(), &rom_stem);
                Ok(())
            })?;

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
        disable_raw_mode()?;

        Ok(())
    }
}
