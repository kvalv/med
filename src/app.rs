use crate::buffer::Buffer;
use std::path::Path;

use crate::{
    cmdbuf::{self},
    event::{AppEvent, Event, EventHandler},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::info;
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
    pub buf: Buffer,

    pub cmdbuf: cmdbuf::CmdBuf,

    // pub cursor: Cursor,
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
        let content: String = std::fs::read_to_string(path.to_str().unwrap()).unwrap();

        Self {
            filename: path.to_str().unwrap().to_string(),
            buf: Buffer::from(content.as_str()),
            mode: Mode::default(),
            // cursor: Cursor { col: 0, row: 0 },
            cmdbuf: cmdbuf::CmdBuf::new(),
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
                    CursorMove(delta_row, delta_col) => {
                        let row = self.buf.row.saturating_add_signed(delta_row as isize);
                        let col = self.buf.col.saturating_add_signed(delta_col as isize);
                        info!("CursorMove: target is row {}, col {}", row, col);
                        self.buf.position(row, col);
                    }
                    ModeChange(mode) => {
                        self.mode = mode;
                    }
                    Write(c) => {
                        // self.buf.position(self.cursor.row, self.cursor.col);
                        self.buf.insert(c);
                    }
                    AdvanceWord => {
                        panic!("No more");
                    }
                    Movement => {
                        let verb = self
                            .cmdbuf
                            .pop()
                            .expect("movement command should have a verb");
                        let count = self.cmdbuf.pop_count().unwrap_or(1);
                        match verb {
                            'w' => {
                                info!("Advance word by {count}");
                                self.buf.advance_word(count as isize);
                            }
                            'b' => {
                                info!("Advance word backwards by {count}");
                                self.buf.advance_word(-(count as isize));
                            }
                            _ => {}
                        }
                    }
                    BufWrite => {
                        std::fs::write(&self.filename, self.buf.text())?;
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
                    KeyCode::Char('x') => {
                        let count = self.cmdbuf.pop_count().unwrap_or(1);
                        self.buf.backspace(count);
                        info!("{count}x backspace");
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
