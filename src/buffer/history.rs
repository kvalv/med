use crate::{buffer::Buffer, span::Span};

pub enum Operation {
    Undo,
    Redo,
}

#[derive(Debug, Clone)]
pub struct Change {
    span: Span,
    old: String,
    new: String,
    // op: Operation,
    // text: String,
}

#[derive(Default, Debug)]
pub struct ChangeHistory {
    // single changes only for now,
    changes: Vec<Change>,
    cursor: Option<usize>,
}

impl ChangeHistory {
    pub fn register(&mut self, span: Span, old: String, new: String) {
        self.changes.push(Change { span, old, new });
        // TODO: when we're not at head
        self.cursor = match self.cursor {
            None => Some(0),
            Some(k) => Some(k + 1),
        }
    }
    pub fn undo(&mut self) -> Option<Change> {
        todo!();
    }
    pub fn redo(&mut self) -> Option<Change> {
        todo!();
    }
}

pub fn apply(buf: &mut Buffer) {}
