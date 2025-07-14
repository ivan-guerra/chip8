use std::time::{Duration, Instant};

use anyhow::anyhow;
use crossterm::{
    event,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use rodio::{source::SineWave, OutputStream, Sink, Source};

use crate::instruction::{decode, Instruction};
use crate::state::{Chip8State, Settings, DISPLAY_HEIGHT, DISPLAY_WIDTH, MEM_SIZE};

const DEFAULT_FREQUENCY: f32 = 440.0;

pub struct Beep {
    sink: Sink,
    #[allow(dead_code)]
    stream: OutputStream,
}

impl Beep {
    pub fn new(freq: f32) -> anyhow::Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        let source = SineWave::new(freq).repeat_infinite();

        sink.append(source);
        sink.pause();

        Ok(Self { sink, stream })
    }

    pub fn on(&mut self) {
        self.sink.play();
    }

    pub fn off(&mut self) {
        self.sink.pause();
    }
}

pub struct Emulator {
    state: Chip8State,
    beeper: Beep,
}

impl Emulator {
    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, rom_name: &str) {
        use ratatui::layout::{Constraint, Direction, Layout};

        // Calculate the exact size needed for 64x32 display plus borders
        let game_width = (DISPLAY_WIDTH as u16) + 2; // +2 for left and right borders
        let game_height = (DISPLAY_HEIGHT as u16) + 2; // +2 for top and bottom borders

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(game_height), // Exact size for game area
                Constraint::Length(7),           // Key mapping area
                Constraint::Min(0),              // Remaining space
            ])
            .split(area);

        // Center the game horizontally if the terminal is wider than needed
        let game_area = if chunks[0].width > game_width {
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(game_width),
                    Constraint::Min(0),
                ])
                .split(chunks[0]);
            horizontal_chunks[1]
        } else {
            chunks[0]
        };

        // Draw main game screen
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

        // Draw key mapping
        let key_mapping = "Key Mapping:\n\
    1 2 3 4    →    1 2 3 C\n\
    Q W E R    →    4 5 6 D\n\
    A S D F    →    7 8 9 E\n\
    Z X C V    →    A 0 B F";
        let key_paragraph = Paragraph::new(key_mapping)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Keypad"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(key_paragraph, chunks[1]);
    }

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

    pub fn new(settings: Settings) -> anyhow::Result<Self> {
        Ok(Emulator {
            state: Chip8State::new(settings),
            beeper: Beep::new(DEFAULT_FREQUENCY)?,
        })
    }

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
