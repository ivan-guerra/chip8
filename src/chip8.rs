use crate::instruction::{Instruction, decode};
use crate::state::{Chip8State, DISPLAY_HEIGHT, DISPLAY_WIDTH, Key, MEM_SIZE, Settings};
use anyhow::anyhow;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::time::{Duration, Instant};

enum AppState {
    Splash,
    Running,
}

pub struct Emulator {
    state: Chip8State,
}

impl Emulator {
    fn draw_frame(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        rom_name: &str,
    ) {
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
        let paragraph = Paragraph::new(row_string)
            .block(Block::default().borders(Borders::ALL).title(rom_name))
            .style(Style::default().fg(Color::White));
        frame.render_widget(paragraph, area);
    }

    fn draw_splash(&self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        let msg = "Welcome to CHIP-8\nPress Enter to start";
        let paragraph = Paragraph::new(msg)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("CHIP-8 Emulator"),
            )
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(paragraph, area);
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

    pub fn new(settings: Settings) -> Self {
        Emulator {
            state: Chip8State::new(settings),
        }
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
        let mut app_state = AppState::Splash;

        'mainloop: loop {
            let frame_start = Instant::now();

            terminal.try_draw(|frame| -> std::io::Result<()> {
                match app_state {
                    AppState::Splash => {
                        self.draw_splash(frame);
                        Ok(())
                    }
                    AppState::Running => {
                        self.state.delay_timer = self.state.delay_timer.saturating_sub(1);
                        self.state.sound_timer = self.state.sound_timer.saturating_sub(1);

                        for _ in 0..=instructions_per_frame {
                            let instruction = self
                                .fetch_instruction()
                                .map_err(|e| std::io::Error::other(e.to_string()))?;
                            instruction
                                .execute(&mut self.state)
                                .map_err(|e| std::io::Error::other(e.to_string()))?;
                        }

                        self.draw_frame(frame, frame.area(), &rom_stem);
                        self.state.keypad.release_key();
                        Ok(())
                    }
                }
            })?;

            if event::poll(std::time::Duration::from_millis(5))? {
                if let Event::Key(key) = event::read()? {
                    match app_state {
                        AppState::Splash => {
                            if key.code == KeyCode::Enter {
                                self.state.reset();
                                self.state.memory.load_rom(&rom_data)?;
                                app_state = AppState::Running;
                            } else if key.code == KeyCode::Esc {
                                terminal.clear()?;
                                break 'mainloop;
                            }
                        }
                        AppState::Running => match key.code {
                            KeyCode::Esc => {
                                terminal.clear()?;
                                break 'mainloop;
                            }
                            KeyCode::Char('1') => {
                                self.state.keypad.press_key(Key::Key1);
                            }
                            KeyCode::Char('2') => {
                                self.state.keypad.press_key(Key::Key2);
                            }
                            KeyCode::Char('3') => {
                                self.state.keypad.press_key(Key::Key3);
                            }
                            KeyCode::Char('4') => {
                                self.state.keypad.press_key(Key::KeyC);
                            }
                            KeyCode::Char('q') => {
                                self.state.keypad.press_key(Key::Key4);
                            }
                            KeyCode::Char('w') => {
                                self.state.keypad.press_key(Key::Key5);
                            }
                            KeyCode::Char('e') => {
                                self.state.keypad.press_key(Key::Key6);
                            }
                            KeyCode::Char('r') => {
                                self.state.keypad.press_key(Key::KeyD);
                            }
                            KeyCode::Char('a') => {
                                self.state.keypad.press_key(Key::Key7);
                            }
                            KeyCode::Char('s') => {
                                self.state.keypad.press_key(Key::Key8);
                            }
                            KeyCode::Char('d') => {
                                self.state.keypad.press_key(Key::Key9);
                            }
                            KeyCode::Char('f') => {
                                self.state.keypad.press_key(Key::KeyE);
                            }
                            KeyCode::Char('z') => {
                                self.state.keypad.press_key(Key::KeyA);
                            }
                            KeyCode::Char('x') => {
                                self.state.keypad.press_key(Key::Key0);
                            }
                            KeyCode::Char('c') => {
                                self.state.keypad.press_key(Key::KeyB);
                            }
                            KeyCode::Char('v') => {
                                self.state.keypad.press_key(Key::KeyF);
                            }
                            _ => {}
                        },
                    }
                }
            }

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
        disable_raw_mode()?;

        Ok(())
    }
}
