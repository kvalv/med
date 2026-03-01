use std::fmt::Display;

use crate::app::App;

pub mod delete;
pub mod movement;
pub mod pattern;
pub mod write;

pub struct CmdBuf {
    buf: Vec<char>,
}

// handlers: vec![
//     // (
//     //     Pattern::try_from("w[rite]").expect("Failed to parse pattern"),
//     //     Box::new(BufWrite {}),
//     // ),
//     (
//         Pattern::try_from("dw").expect("Failed to parse pattern"),
//         Box::new(delete::Delete {}),
//     ),
//     // (
//     //     Pattern::try_from("w[rite]").expect("Failed to parse pattern"),
//     //     Box::new(write::BufWrite {}),
//     // ),
// ],

/// A buffer that holds the current command being typed by the user.
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

    // pops from the cmdbuf a count
    pub fn pop_count(&mut self) -> Option<usize> {
        let count: String = self.buf.iter().take_while(|c| c.is_numeric()).collect();

        if count.is_empty() {
            None
        } else {
            for _ in 0..count.len() {
                self.buf.remove(0);
            }
            count.parse::<usize>().ok()
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

pub trait CmdHandler: Display {
    fn handle(&self, app: &mut App);
}

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

#[cfg(test)]
mod tests;
