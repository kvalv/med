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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    pub start: Location,
    pub end: Location,
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
