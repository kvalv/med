use crate::app::App;
use crate::buffer::Buffer;
use crate::cmd::movement::movement;
use crate::span::{Position, Span};
use crate::textobject::{Boundary, TextObject};
use crate::wordclass::WordClass;
use chumsky::{Parser, error::Rich, extra, prelude::*};
use log::info;

pub mod append;
pub mod change;
pub mod insert;
pub mod movement;
pub mod undo;
pub mod write;

pub struct CmdBuf {
    buf: Vec<char>,
}

impl CmdBuf {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }
    pub fn push(&mut self, c: char) {
        self.buf.push(c);
    }
    pub fn drain(&mut self) -> std::vec::Drain<'_, char> {
        self.buf.drain(..)
    }
    pub fn last(&self) -> Option<char> {
        self.buf.last().cloned()
    }
    pub fn pop(&mut self) -> Option<char> {
        self.buf.pop()
    }
    pub fn pop_left(&mut self) -> Option<char> {
        if self.buf.is_empty() {
            None
        } else {
            Some(self.buf.remove(0))
        }
    }

    pub fn parse(&mut self) -> Option<Command> {
        let input: String = self.buf.iter().collect();
        parser().parse(&input).output().map(|cmd| {
            self.buf.clear();
            cmd.clone()
        })
    }

    // pops from the cmdbuf a count
    pub fn pop_count(&mut self, default_value: usize) -> usize {
        let count: String = self.buf.iter().take_while(|c| c.is_numeric()).collect();

        if count.is_empty() {
            default_value
        } else {
            for _ in 0..count.len() {
                self.buf.remove(0);
            }
            count.parse::<usize>().unwrap_or(default_value)
        }
    }
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn text(&self) -> String {
        self.buf.iter().collect()
    }

    pub fn count(&mut self) -> usize {
        if self.buf.is_empty() {
            1
        } else {
            self.buf
                .drain(..)
                .collect::<String>()
                .parse::<usize>()
                .unwrap_or(1)
        }
    }
}

impl Default for CmdBuf {
    fn default() -> Self {
        Self::new()
    }
}

pub type CommandHandler = fn(app: &mut App) -> Result<(), String>;

impl std::fmt::Debug for CmdBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CmdBuf {{ buf: {:?} }}", self.buf)
    }
}

impl std::fmt::Display for CmdBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.buf.iter().collect::<String>())
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Delete {
    count: Option<usize>,
    text_object: Option<(Boundary, TextObject)>,
    movement: Option<Movement>, // delete by movemeng, e.g. 'dj' or 'dG'
    linewise: bool,             // dd
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Movement {
    count: Option<usize>,
    char: char, // hjkl, 0, $ ,...
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Invalid,
    Movement(Movement),
    Delete(Delete),
}

impl Command {
    pub fn execute(&self, app: &mut App) -> Result<(), String> {
        match self {
            Command::Movement(m) => movement(app, m),
            Command::Delete(d) => {
                let span = if let Some((boundary, text_object)) = d.text_object {
                    app.buf
                        .span_for_textobject(text_object, boundary, d.count.unwrap_or(1))
                } else if let Some(movement) = &d.movement {
                    movement.span(&mut app.buf)
                } else if d.linewise {
                    todo!();
                } else {
                    todo!();
                };

                let old = app.buf.delete_span(span, true);
                let change = crate::buffer::history::Change {
                    span,
                    old,
                    new: "".to_string(),
                };
                app.buf.register_change(change);

                // Placeholder for delete execution logic
                info!("Executing delete: {:?}", d);
                Ok(())
            }
            Command::Invalid => Err("Invalid command".to_string()),
        }
    }
}

// fn parse<'a>() -> impl Parser<'a, &'a str, Command, extra::Err<Rich<'a, char>>> {
fn parser<'a>() -> impl Parser<'a, &'a str, Command, extra::Err<Rich<'a, char>>> {
    let count = text::digits(10)
        .to_slice()
        .map(|d: &str| d.parse::<usize>().expect("not a number"));

    // // aw, ae, iw, ie
    let text_object = one_of("ai").then(one_of("wWe()[]{}<>")).map(|(n, c)| {
        let boundary = match n {
            'a' => Boundary::Around,
            _ => Boundary::Inner,
        };
        let text_object = match c {
            'w' => TextObject::Word,
            'e' => TextObject::WordEnd,
            '(' => TextObject::Paren,
            ')' => TextObject::Paren,
            '{' => TextObject::CurlyBracket,
            '}' => TextObject::CurlyBracket,
            _ => todo!(),
        };
        (boundary, text_object)
    });

    let movement0 = just('0').to(Movement {
        count: None,
        char: '0',
    });
    let movement1 = count
        .or_not()
        .then(one_of("hjkl0$GwWbBeE"))
        .map(|(count, char)| Movement { count, char });
    let movement = movement0.or(movement1);

    let delete_by_text_object = // d4j
            count
                .or_not()
                .then_ignore(just('d'))
                .then(text_object)
                .map(|(count, text_object)| Delete {
                    count,
                    text_object: Some(text_object),
                    movement: None,
                    linewise: false,
                });
    let delete_by_movement = // diw
            count
                .or_not()
                .then_ignore(just('d'))
                .then(movement)
                .map(|(count, m)| Delete {
                    count,
                    text_object: None,
                    movement: Some(m),
                    linewise: false,
                });
    let delete = delete_by_movement.or(delete_by_text_object);

    choice((
        delete.map(Command::Delete),
        movement.map(Command::Movement),
        // Placeholder for more...
    ))
    .recover_with(via_parser(nested_delimiters(
        '{',
        '}',
        [('[', ']')],
        |_| Command::Invalid,
    )))
}

impl Movement {
    pub fn span(&self, buf: &mut Buffer) -> Span {
        // ... right?
        // word -> go until we are start of next word
        // let mut span = Span::empty_at(b.row, b.col);
        let mut index = buf.c;
        let p = buf.current_position();
        let count = self.count.unwrap_or(1);

        match self.char {
            'w' => {
                let start = WordClass::from(buf.current_char());
                for _ in 0..count {
                    if start == WordClass::WHITESPACE {
                        // eat whitespace
                        index = advance_while(index, buf, WordClass::WHITESPACE);
                    } else {
                        index = advance_while(index, buf, start);
                        if let Some(c) = buf.char_at(index)
                            && WordClass::from(c) == WordClass::WHITESPACE
                        {
                            index = advance_while(index, buf, WordClass::WHITESPACE)
                        }
                    }
                }
            }
            'b' => {
                let mut start = WordClass::from(buf.current_char());
                for _ in 0..count {
                    index = index.saturating_sub(1); // always one back, unless start of file
                    if start != WordClass::WHITESPACE {
                        // eat whitespace
                        index = back_while(index, buf, WordClass::WHITESPACE);
                    }

                    start = WordClass::from(buf.char_at(index).unwrap());

                    index = back_while(index, buf, start);
                    // only increment if we're on a non-matching char
                    if let Some(c) = buf.char_at(index)
                        && WordClass::from(c) != start
                    {
                        index += 1; // first non-matching char -> advance one
                    }
                }
            }
            'j' => {
                let num_rows = buf.rows_total();
                let row = p.row + (count).min(num_rows - 1 - p.row);
                let col = buf
                    .target_col
                    .unwrap_or(buf.current_position().col)
                    .min(buf.num_columns(row));
                index = buf.to_index(Position { row, col }).unwrap();
            }
            'k' => {
                let row = p.row.saturating_sub(count);
                info!("Moving up to row {}, current row {}", row, p.row);
                let col = buf
                    .target_col
                    .unwrap_or(buf.current_position().col)
                    .min(buf.num_columns(row));
                index = buf.to_index(Position { row, col }).unwrap();
            }
            'h' => {
                let col = p.col - count.min(p.col);
                index = buf.to_index(Position { row: p.row, col }).unwrap();
            }
            'l' => {
                let col = (p.col + count).min(buf.num_columns(p.row));
                index = buf.to_index(Position { row: p.row, col }).unwrap();
            }
            '$' => {
                let col = buf.num_columns(p.row);
                index = buf.to_index(Position { row: p.row, col }).unwrap();
            }
            '0' => {
                index = buf.to_index(Position { row: p.row, col: 0 }).unwrap();
            }
            _ => todo!(),
        }

        let span = Span {
            start: buf.current_position(),
            end: buf.to_position(index).unwrap(),
        };
        info!("Movement span: {}", span);
        span
    }
}

fn advance_while(mut index: usize, buf: &Buffer, wc: WordClass) -> usize {
    loop {
        match buf.char_at(index) {
            Some(c) => {
                if WordClass::from(c) == wc {
                    index += 1
                } else {
                    return index;
                }
            }
            None => return index,
        }
    }
}

// goes back until the condition no longer holds. Then returns
// ... meaning we are at the position of the first char that does not match
fn back_while(mut index: usize, buf: &Buffer, wc: WordClass) -> usize {
    loop {
        if index == 0 {
            return index;
        }
        match buf.char_at(index) {
            Some(c) => {
                if WordClass::from(c) == wc {
                    index -= 1
                } else {
                    return index;
                }
            }
            None => return index,
        }
    }
}

#[cfg(test)]
mod tests;
