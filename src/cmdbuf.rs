#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
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

    pub fn count(&mut self) -> usize {
        if self.buf.len() == 0 {
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
