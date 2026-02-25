use std::path::Path;

use crate::{
    cmdbuf::{self},
    event::{AppEvent, Event, EventHandler},
    movement::next_word,
};
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
    Command,
}
impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Command => write!(f, "COMMAND"),
        }
    }
}

/// Application.
#[derive(Debug)]
pub struct App {
    // Location of the cursor
    pub filename: String,
    pub buf: Vec<String>,

    pub cmdbuf: cmdbuf::CmdBuf,

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
    pub fn new(path: &Path) -> Self {
        Self {
            filename: path.to_str().unwrap().to_string(),
            buf: std::fs::read_to_string(path.to_str().unwrap())
                .unwrap()
                .lines()
                .map(|str| str.to_string())
                .collect(),
            mode: Mode::default(),
            cursor: Cursor { col: 0, row: 0 },
            cmdbuf: cmdbuf::CmdBuf::new(),
            running: true,
            counter: 0,
            events: EventHandler::new(),
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        use AppEvent::*;

        let row = self.cursor.row;
        let col = self.cursor.col;

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
                        if let Some(line) = self.buf.get_mut(self.cursor.row) {
                            line.insert(self.cursor.col, c);
                        }
                        self.cursor.col += 1;
                    }
                    AdvanceWord => {
                        panic!("No more");
                        let count = self.cmdbuf.count();
                        if count > 1 {
                            todo!("Advance by n words {count}");
                        }
                        let j = next_word(&self.buf[self.cursor.row], self.cursor.col, count);
                        if let Some(j) = j {
                            self.cursor.col = j;
                        } else {
                            self.cursor.col = self.buf[self.cursor.row].len();
                        }
                    } // _ => todo!(),
                    Movement => {
                        let verb = self
                            .cmdbuf
                            .pop()
                            .expect("movement command should have a verb");
                        let count = self.cmdbuf.pop_count().unwrap_or(1);
                        match verb {
                            'w' => {
                                self.cursor.col =
                                    next_word(&self.buf[self.cursor.row], self.cursor.col, count)
                                        .unwrap_or_else(|| self.buf[self.cursor.row].len());
                            }
                            _ => {}
                        }
                    }
                    BufWrite => {
                        std::fs::write(&self.filename, self.buf.join("\n"))?;
                    }
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

        match self.mode {
            // if self.mode == Mode::Normal {
            Mode::Insert => match key_event.code {
                KeyCode::Esc => self.events.send(AppEvent::ModeChange(Mode::Normal)),
                KeyCode::Char(c) => self.events.send(AppEvent::Write(c)),
                _ => {}
            },
            Mode::Normal => {
                match key_event.code {
                    KeyCode::Char('i') => self.events.send(AppEvent::ModeChange(Mode::Insert)),
                    KeyCode::Char(':' | ';') => {
                        self.events.send(AppEvent::ModeChange(Mode::Command))
                    }
                    KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                    KeyCode::Char('j') => self.events.send(AppEvent::CursorMove(1, 0)),
                    KeyCode::Char('k') => self.events.send(AppEvent::CursorMove(-1, 0)),
                    KeyCode::Char('h') => self.events.send(AppEvent::CursorMove(0, -1)),
                    KeyCode::Char('l') => self.events.send(AppEvent::CursorMove(0, 1)),
                    KeyCode::Char('0') => self.events.send(AppEvent::CursorMove(0, -9999)),
                    // KeyCode::Char('w') => self.events.send(AppEvent::AdvanceWord),
                    KeyCode::Char(c @ ('w' | 'b')) => {
                        self.cmdbuf.push(c);
                        self.events.send(AppEvent::Movement);
                    }

                    // TODO: w, ...
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        // fill up the command buffer
                        self.cmdbuf.push(c);
                    }
                    _ => {}
                }
                // } else if self.mode == Mode::Insert {
            }
            _ => match key_event.code {
                KeyCode::Enter => {
                    match self.cmdbuf.drain().collect::<String>().as_str() {
                        "w" => self.events.send(AppEvent::BufWrite),
                        w => todo!("implement cmd for {w}"),
                    }
                    // todo!("Handle enter key in command mode");
                }
                KeyCode::Esc => {
                    self.cmdbuf = cmdbuf::CmdBuf::new();
                    self.events.send(AppEvent::ModeChange(Mode::Normal));
                }
                KeyCode::Char(c) => {
                    self.cmdbuf.push(c);
                }
                _ => todo!("Handle other keys in command mode"),
            },
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
