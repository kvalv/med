#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.row, self.col)
    }
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self {
            row: value.0,
            col: value.1,
        }
    }
}

impl std::ops::Add<Span> for Position {
    type Output = Position;

    fn add(self, rhs: Span) -> Self::Output {
        Position {
            row: self.row + rhs.delta_rows(),
            col: if rhs.delta_rows() > 0 {
                rhs.end.col
            } else {
                self.col + rhs.end.col
            },
        }
    }
}

/// End last column not included
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

// displa (a,b) (c, d)
impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({},{}), ({},{})",
            self.start.row, self.start.col, self.end.row, self.end.col
        )
    }
}

impl Span {
    pub fn empty_at(row: usize, col: usize) -> Self {
        Self {
            start: Position { row, col },
            end: Position { row, col },
        }
    }

    pub fn delta_rows(&self) -> usize {
        self.end.row - self.start.row
    }
}

impl From<(usize, usize, usize, usize)> for Span {
    fn from(value: (usize, usize, usize, usize)) -> Self {
        Self {
            start: Position {
                row: value.0,
                col: value.1,
            },
            end: Position {
                row: value.2,
                col: value.3,
            },
        }
    }
}

impl From<&str> for Span {
    fn from(value: &str) -> Self {
        let mut s = Self {
            start: Position { row: 0, col: 0 },
            end: Position { row: 0, col: 0 },
        };
        for c in value.chars() {
            if c == '\n' {
                s.end.row += 1;
                s.end.col = 0;
            } else {
                s.end.col += 1;
            }
        }
        s
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
            start: Position {
                row: self.start.row,
                col: self.start.col + n,
            },
            end: Position {
                row: self.end.row,
                col: self.end.col - n,
            },
        }
    }
}
