use crate::app::App;

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct CmdBuf {
    buf: Vec<char>,
}

impl std::fmt::Display for CmdBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.buf.iter().collect::<String>())
    }
}

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
        self.buf
            .iter()
            .rev()
            .take_while(|c| c.is_digit(10))
            .collect::<String>()
            .parse::<usize>()
            .ok()
    }
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
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

    pub fn register<F>(&mut self, pat: &str, mut callback: F)
    where
        F: FnMut(&mut App),
    {
        todo!();
    }
}

/// Describes a comand pattern, e.g. `w[rite]` matches both `w` and `wr`, ...
pub struct Pattern(Vec<Criteria>);

/// A single letter in the command. Either encapsulated in a `[...]` or not.
/// If it is, then it's optional. Otherwise, it's required.
struct Criteria {
    c: char,
    required: bool,
}

impl TryFrom<&str> for Pattern {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut v = vec![];
        let mut required = true;
        for c in value.chars() {
            if c == '[' {
                required = false;
            } else if c == ']' {
                required = true;
            } else {
                v.push(Criteria { c, required });
            }
        }

        Ok(Self(v))
    }
}

impl Pattern {
    pub fn matches(&self, test: &str) -> bool {
        // w[rite] matches w, wr, wri, writ, write
        for (i, c) in test.chars().enumerate() {
            match self.0.get(i) {
                None => return false,
                Some(crit) => {
                    if c != crit.c {
                        return false;
                    }
                    // otherwise they are the same
                }
            }
        }

        // we made it here. That means all match. We check that there are no required
        // input left
        let rest_optional = self.0.iter().skip(test.len()).all(|crit| !crit.required);
        rest_optional
    }
}

#[cfg(test)]
mod tests;
