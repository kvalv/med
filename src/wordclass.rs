#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WordClass(pub u8);

impl WordClass {
    pub const ALPHANUMERIC: WordClass = WordClass(1);
    pub const WHITESPACE: WordClass = WordClass(2);
    pub const SYMBOLS: WordClass = WordClass(4);
    pub const NEWLINE: WordClass = WordClass(8);

    pub fn contains(self, other: WordClass) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for WordClass {
    type Output = WordClass;
    fn bitor(self, rhs: WordClass) -> WordClass {
        WordClass(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for WordClass {
    type Output = WordClass;
    fn bitand(self, rhs: WordClass) -> WordClass {
        WordClass(self.0 & rhs.0)
    }
}

impl From<char> for WordClass {
    fn from(value: char) -> Self {
        if value.is_alphanumeric() || value == '_' {
            Self::ALPHANUMERIC
        } else if value == '\n' {
            Self::NEWLINE
        } else if value == ' ' || value == '\t' {
            Self::WHITESPACE
        } else {
            Self::SYMBOLS
        }
    }
}
