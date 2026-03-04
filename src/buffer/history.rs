use crate::span::Span;

pub enum Operation {
    Undo,
    Redo,
}

#[derive(Debug, Clone)]
pub struct Change {
    pub span: Span,
    pub old: String,
    pub new: String,
    // op: Operation,
    // text: String,
}

impl Change {
    pub fn flip(&self) -> Self {
        Self {
            span: self.span,
            old: self.new.clone(),
            new: self.old.clone(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ChangeHistory {
    // single changes only for now,
    changes: Vec<Change>,

    // 0 means we have no changes (empty). 1 means 1 change and so on.
    // if changes.len() is 4 and we are at 0, then that means we have some changes but
    // they're all reset
    cursor: usize,
}

impl ChangeHistory {
    pub fn register(&mut self, change: Change) {
        self.changes.drain(self.cursor..);
        self.changes.push(change);
        self.cursor += 1;
        assert!(self.cursor <= self.changes.len());
    }

    pub fn undo(&mut self) -> Option<Change> {
        if self.cursor == 0 {
            None
        } else {
            self.cursor -= 1;
            assert!(self.cursor <= self.changes.len());
            Some(self.changes[self.cursor].clone())
        }
    }
    pub fn redo(&mut self) -> Option<Change> {
        if self.cursor >= self.changes.len() - 1 {
            None
        } else {
            self.cursor += 1;
            assert!(self.cursor <= self.changes.len());
            Some(self.changes[self.cursor].clone())
        }
    }
}
