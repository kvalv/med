use crate::{
    buffer::Buffer,
    cmd::{
        CommandHandler,
        pattern::{MatchResult, Pattern},
    },
};
use std::path::Path;

use crate::{
    cmd::{self},
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
    ExCommand,
}
impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::ExCommand => write!(f, "EXCOMMAND"),
        }
    }
}

/// Application.
// #[derive(Debug)]
pub struct App {
    // Location of the cursor
    pub filename: String,
    pub buf: Buffer,

    pub cmdbuf: cmd::CmdBuf,

    pub msg: Option<String>,

    // pub cursor: Cursor,
    pub mode: Mode,

    /// Is the application running?
    pub running: bool,
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
            msg: None,
            mode: Mode::default(),
            // cursor: Cursor { col: 0, row: 0 },
            cmdbuf: cmd::CmdBuf::new(),
            running: true,
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
                    ModeChange(mode) => {
                        if self.mode == Mode::Insert && mode == Mode::Normal {
                            self.buf.left(1); // move cursor back one left
                        }
                        if mode == Mode::ExCommand || mode == Mode::Insert {
                            self.clear_msg();
                        }
                        self.mode = mode;
                        self.buf.clear_target_col();
                    }
                    Write(c) => {
                        // self.buf.position(self.cursor.row, self.cursor.col);
                        // self.buf.left(1);
                        self.buf.insert(c);
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
                                self.buf.w(count);
                                self.buf.clear_target_col();
                            }
                            'b' => {
                                info!("Advance word backwards by {count}");
                                self.buf.b(count);
                                self.buf.clear_target_col();
                            }
                            'e' => {
                                info!("Advance to end of word by {count}");
                                self.buf.e(count);
                            }
                            'h' => {
                                info!("Move left by {count}");
                                self.buf.h(count);
                            }
                            'j' => {
                                info!("Move down by {count}");
                                self.buf.j(count);
                            }
                            'k' => {
                                info!("Move up by {count}");
                                self.buf.k(count);
                            }
                            'l' => {
                                info!("Move right by {count}");
                                self.buf.l(count);
                            }
                            '0' => {
                                info!("Move to beginning of line");
                                self.buf.position(self.buf.row, 0);
                                self.buf.clear_target_col();
                            }
                            _ => {}
                        }
                    }
                    BufWrite => {
                        std::fs::write(&self.filename, self.buf.text())?;
                    }
                    ExCommandSubmit => {
                        let cmd = self.cmdbuf.drain().collect::<String>();
                        info!("Command submitted: {}", cmd);
                        match cmd.as_str() {
                            "w" | "wr" | "wri" | "writ" | "write" => {
                                self.events.send(AppEvent::BufWrite);
                                self.set_msg(format!("Wrote to file {}", self.filename));
                            }
                            "q" | "qu" | "qui" | "quit" => {
                                self.events.send(AppEvent::Quit);
                            }
                            "wq" => {
                                self.events.send(AppEvent::BufWrite);
                                self.events.send(AppEvent::Quit);
                            }
                            _ => {
                                info!("Unknown command: {}", cmd);
                            }
                        }

                        self.events.send(AppEvent::ModeChange(Mode::Normal));
                    }
                },
            }
        }
        Ok(())
    }

    fn set_msg(&mut self, msg: String) {
        self.msg = Some(msg);
        info!("Message set: {}", self.msg.as_ref().unwrap());
    }
    fn clear_msg(&mut self) {
        self.msg = None;
        info!("Message cleared");
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
                KeyCode::Enter => self.events.send(AppEvent::Write('\n')),
                KeyCode::Backspace => self.buf.backspace(1),
                _ => {}
            },
            Mode::Normal => {
                match key_event.code {
                    // KeyCode::Char('i') => self.events.send(AppEvent::ModeChange(Mode::Insert)),
                    KeyCode::Char(':' | ';') => {
                        self.events.send(AppEvent::ModeChange(Mode::ExCommand))
                    }
                    KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                    KeyCode::Char('$') => {
                        self.buf.eol();
                    }
                    KeyCode::Char(c @ ('b' | 'e' | 'h' | 'j' | 'k' | 'l' | '0')) => {
                        self.cmdbuf.push(c);
                        self.events.send(AppEvent::Movement);
                    }
                    KeyCode::Char('x') => {
                        let count = self.cmdbuf.pop_count().unwrap_or(1);
                        self.buf.x(count);
                        info!("{count}x backspace");
                    }
                    // KeyCode::Char('A') => {
                    //     self.buf.eol();
                    //     self.buf.right(1);
                    //     self.events.send(AppEvent::ModeChange(Mode::Insert));
                    // }
                    // KeyCode::Char('a') => {
                    //     self.buf.right(1);
                    //     self.events.send(AppEvent::ModeChange(Mode::Insert));
                    // }
                    KeyCode::Backspace => {
                        self.buf.h(1);
                    }
                    KeyCode::Esc => {
                        self.cmdbuf.drain();
                    }
                    KeyCode::Char('o') => {
                        self.buf.eol();
                        self.buf.l(1);
                        self.buf.insert('\n');
                        self.events.send(AppEvent::ModeChange(Mode::Insert));
                    }

                    // TODO: w, ...
                    KeyCode::Char(c) => {
                        // fill up the command buffer
                        self.cmdbuf.push(c);
                        info!(
                            "Pushed '{c}' to command buffer, now: {}",
                            self.cmdbuf.text()
                        );

                        let cmd = self.cmdbuf.text();
                        match create_command_handlers()
                            .into_iter()
                            .map(|(pat, handler)| (pat.matches(&cmd), handler))
                            .max_by(|x, y| x.0.cmp(&y.0))
                            .unwrap()
                        {
                            (MatchResult::Match, handler) => {
                                info!("Command matched pattern '{}'", cmd);
                                let _ = handler(self);
                                self.cmdbuf.drain();
                            }
                            (MatchResult::NoMatch, _) => {
                                // Regardless of what gets typed next, it's not going to match
                                // anything. In that case, we'll clear the cmdbuf
                                info!("Nothing will match '{}' -> clearing cmdbuf", cmd);
                                self.cmdbuf.drain();
                            }
                            (MatchResult::PartialMatch, _) => {
                                // Still hope for matching -> keep
                            }
                        }
                    }
                    _ => {}
                }
            }
            Mode::ExCommand => match key_event.code {
                KeyCode::Enter => {
                    self.events.send(AppEvent::ExCommandSubmit);
                }
                KeyCode::Esc => {
                    self.cmdbuf = cmd::CmdBuf::new();
                    self.events.send(AppEvent::ModeChange(Mode::Normal));
                }
                KeyCode::Char(c) => {
                    self.cmdbuf.push(c);
                }
                _ => {}
            },
            #[allow(unreachable_patterns)]
            _ => panic!("Unknown mode"),
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
}

type CommandHandlers = Vec<(Pattern, CommandHandler)>;

fn create_command_handlers() -> CommandHandlers {
    vec![
        (Pattern::from("i"), cmd::insert::insert),
        (Pattern::from("a") | Pattern::from("A"), cmd::append::append),
        (Pattern::from("d<motion>"), cmd::delete::delete),
        (Pattern::from("c<motion>"), cmd::change::change),
        (
            Pattern::from("<count>w") | Pattern::from("<count>e"),
            cmd::movement::movement,
        ),
    ]
}
