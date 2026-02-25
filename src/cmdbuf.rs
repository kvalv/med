#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct CmdBuf {
    buf: Vec<char>,
}

impl std::fmt::Display for CmdBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.buf.iter().collect::<String>())
    }
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
