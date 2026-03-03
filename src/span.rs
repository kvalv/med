#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl From<(usize, usize)> for Location {
    fn from(value: (usize, usize)) -> Self {
        Self {
            row: value.0,
            col: value.1,
        }
    }
}

/// End last column not included
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Span {
    pub fn empty_at(row: usize, col: usize) -> Self {
        Self {
            start: Location { row, col },
            end: Location { row, col },
        }
    }
}

impl From<(usize, usize, usize, usize)> for Span {
    fn from(value: (usize, usize, usize, usize)) -> Self {
        Self {
            start: Location {
                row: value.0,
                col: value.1,
            },
            end: Location {
                row: value.2,
                col: value.3,
            },
        }
    }
}

impl Span {
    pub fn is_empty(&self) -> bool {
        self.start.row == self.end.row && self.start.col == self.end.col
    }

    // shrink at both sides
    pub fn shrink(&self, n: usize) -> Self {
        if self.end.col < n {
            panic!("Cannot shrink span by {} characters", n);
        }
        Self {
            start: Location {
                row: self.start.row,
                col: self.start.col + n,
            },
            end: Location {
                row: self.end.row,
                col: self.end.col - n,
            },
        }
    }
}
