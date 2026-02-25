use std::path::{Path, PathBuf};

use crate::event::{AppEvent, Event, EventHandler};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Cursor {
    pub col: usize,
    pub row: usize,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

/// Application.
#[derive(Debug)]
pub struct App {
    // Location of the cursor
    pub buf: String,

    pub cursor: Cursor,

    pub mode: Mode,

    /// Is the application running?
    pub running: bool,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(path: &PathBuf) -> Self {
        Self {
            buf: std::fs::read_to_string(path.clone().to_str().unwrap()).expect("failed to read"),
            mode: Mode::default(),
            cursor: Cursor { col: 0, row: 0 },
            running: true,
            counter: 0,
            events: EventHandler::new(),
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        use AppEvent::*;

        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        self.handle_key_events(key_event)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    Quit => self.quit(),
                    CursorMove(i, j) => {
                        self.cursor = Cursor {
                            row: if i > 0 {
                                self.cursor.row.saturating_add(i as usize)
                            } else {
                                self.cursor.row.saturating_sub(-i as usize)
                            },
                            col: if j > 0 {
                                self.cursor.col.saturating_add(j as usize)
                            } else {
                                self.cursor.col.saturating_sub(-j as usize)
                            },
                        }
                    }
                    ModeChange(mode) => {
                        self.mode = mode;
                    }
                    Write(c) => {
                        self.buf = self
                            .buf
                            .lines()
                            .enumerate()
                            .map(|(i, line)| {
                                if (self.cursor.row == i) {
                                    let mut tmp = line.to_string();
                                    tmp.insert(self.cursor.col, c);
                                    tmp
                                } else {
                                    line.to_string()
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("\n");
                        self.cursor.col += 1;
                    }
                    _ => todo!(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        // Global keybindings (any mode)
        match key_event.code {
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit);
                return Ok(());
            }
            _ => {}
        }

        if self.mode == Mode::Normal {
            match key_event.code {
                KeyCode::Char('j') => self.events.send(AppEvent::CursorMove(1, 0)),
                KeyCode::Char('k') => self.events.send(AppEvent::CursorMove(-1, 0)),
                KeyCode::Char('h') => self.events.send(AppEvent::CursorMove(0, -1)),
                KeyCode::Char('l') => self.events.send(AppEvent::CursorMove(0, 1)),
                KeyCode::Char('i') => self.events.send(AppEvent::ModeChange(Mode::Insert)),
                KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Char('0') => self.events.send(AppEvent::CursorMove(0, -9999)),
                // TODO: respect word boundaries
                KeyCode::Char('$') => self.events.send(AppEvent::CursorMove(0, 9999)),
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Esc => self.events.send(AppEvent::ModeChange(Mode::Normal)),
                KeyCode::Char(c) => self.events.send(AppEvent::Write(c)),
                _ => {}
            }
        }

        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }
}
