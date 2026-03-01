// Gap buffer:
// B, C, CE and E
// prefix: B -> C
// suffic: CE -> E
//
// moving left -> keep gap size constant -> has to shift one element to the right set
// moving right: similar as above
// write: just insert, unless need to grow. if need grow -> grow, then insert
// remove? move pointer to the left.

use std::char;

use log::{info, warn};

use crate::textobject::{Boundary, TextObject};

/// A Buffer represents a file that is being edited.
/// Implemented as a gap buffer
#[derive(Debug)]
pub struct Buffer {
    buf: Vec<char>,
    c: usize,
    d: usize,

    pub row: usize,
    pub col: usize,

    // when moving up or donw, the target column is what the desired column is to be at given
    // enough caracters, when doing jjjj or kkkk
    target_col: Option<usize>,
}

// Display
impl std::fmt::Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}

impl From<&str> for Buffer {
    fn from(value: &str) -> Self {
        let d = value.chars().count().saturating_mul(3) / 2;
        let mut buffer = Self::new(d);

        // let mut buf = vec!['x'; d];
        for c in value.chars() {
            buffer.insert(c);
        }
        buffer.position(0, 0);
        buffer
    }
}

impl Buffer {
    fn new(cap: usize) -> Self {
        Self {
            row: 0,
            col: 0,
            target_col: None,
            buf: vec!['x'; cap],
            c: 0,
            d: cap,
        }
    }
    pub fn text(&self) -> String {
        self.buf
            .iter()
            .take(self.c)
            .chain(self.buf.iter().skip(self.d))
            .collect()
    }

    pub fn h(&mut self, size: usize) {
        if self.col == 0 {
            return;
        }
        self.left(size);
        self.target_col = None;
    }
    pub fn l(&mut self, size: usize) {
        if self.next_char().map(|c| c == '\n').unwrap_or(true) {
            return;
        }
        self.right(size);
        self.target_col = None;
    }

    /// Move left
    pub fn left(&mut self, count: usize) {
        for _ in 0..count {
            if self.c == 0 {
                return;
            }

            // d receives content from left side, then decrement
            let recv = self.buf[self.c - 1];

            self.buf[self.d - 1] = recv;

            if recv == '\n' {
                self.row = self.row.saturating_sub(1);
                // we need to recalculate col by counting chars until the next newline
                // this is almost certainly wrong

                self.col = 0;
                for i in (0..self.c - 1).rev() {
                    if self.buf[i] == '\n' {
                        break;
                    }
                    self.col += 1;
                }
            } else {
                self.col = self.col.saturating_sub(1);
            }

            // Then shift pointers
            self.c -= 1;
            self.d -= 1;
        }
    }

    /// Move right
    pub fn right(&mut self, count: usize) {
        for _ in 0..count {
            if self.d == self.buf.len() - 1 {
                return;
            }

            // c receives content from right side, then increment
            let recv = self.buf[self.d];
            self.buf[self.c] = recv;

            if recv == '\n' {
                self.row += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }

            // Then shift pointers
            self.d += 1;
            self.c += 1;
        }
    }

    fn advance_while<T: Fn(char) -> bool>(&self, start: usize, cond: T) -> usize {
        let mut i = 0;
        loop {
            if start + i >= self.buf.len() {
                return start + i;
            }
            let c = self.buf[start + i];
            if !cond(c) {
                return start + i;
            }
            i += 1;
        }
    }

    pub fn d(&mut self, count: usize, boundary: Boundary, obj: TextObject) {
        use Boundary::*;
        use TextObject::*;
        match (boundary, obj) {
            (Current, Word) => {
                for _ in 0..count {
                    // munch words until we're at
                    let xx = self.advance_while(self.d, |c| is_word(c));
                    self.d = self.advance_while(xx, is_blank);

                    if self.is_eol() {
                        // If we are at the end of the line, we want to
                        // move the cursor left a bit
                        self.left(1);
                    }
                }
            }
            (Inner, Word) => {
                for i in 0..100 {
                    if !is_word(self.buf[self.c - i]) {
                        // ... we are done.
                        self.c -= i;
                        break;
                    }
                }

                for i in 0..100 {
                    if !is_word(self.buf[self.d + i]) {
                        self.d += i;
                        break;
                    }
                }
            }
            (Around, Word) => {}

            _ => todo!(),
        }
    }

    /// Moves to a particular line
    pub fn position(&mut self, row: usize, col: usize) {
        let num_rows = self.text().lines().count();
        if row > num_rows {
            warn!("Not yet implemented: go outside of file");
            return;
        }

        // println!("Starting position {}, {} ", self.row, self.col);

        for _ in 0..100 {
            // println!(
            //     "Position: {},{} target {},{} - next '{}'",
            //     self.row,
            //     self.col,
            //     row,
            //     col,
            //     self.next_char().unwrap_or('_'),
            //     // self.current_char()
            // );

            if self.row < row {
                self.right(1);
            } else if self.row > row {
                self.left(1);
            } else if self.col < col && self.next_char().map(|c| c != '\n').unwrap_or(false) {
                self.right(1);
            } else if self.col > col {
                self.left(1);
            } else {
                return;
            }
        }
        warn!("Max iterations reached");
    }

    fn is_eol(&self) -> bool {
        self.next_char().is_none() || self.next_char() == Some('\n')
    }

    /// Inserts to the left of the cursor.
    pub fn insert(&mut self, v: char) {
        if self.c == self.d {
            self.grow();
        }
        self.buf[self.c] = v;
        self.c += 1;

        if v == '\n' {
            self.row += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
    }

    pub fn backspace(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        for _ in 0..count {
            if self.c == 0 {
                return;
            }
            self.c -= 1;
        }
    }
    fn grow(&mut self) {
        // we record how far off from the right side d is
        let offset = self.buf.len() - self.d;

        // I'm sure there's a better way to do this
        let mut buf = vec!['x'; self.buf.len() * 2];
        for (i, v) in self.buf.drain(0..self.c).enumerate() {
            buf[i] = v;
        }
        let n = buf.len();
        let m = self.buf.len();
        for (i, v) in self.buf.drain(..).enumerate() {
            buf[n - m + i] = v;
        }
        self.d = n - offset;
        self.buf = buf;
    }

    pub fn x(&mut self, count: usize) {
        for _ in 0..count {
            if self.current_char() == '\n' {
                self.backspace(1);
            } else {
                self.d += 1;
            }
        }
    }

    /// vim-motion, go forward by word
    pub fn w(&mut self, count: usize) {
        if count == 0 {
            info!(
                "count is 0, doing nothing - current char '{}' at index {}",
                self.current_char(),
                self.c,
            );
            return;
        }

        let mut words_visited = 0;

        const LIMIT: usize = 20;

        for _ in 0..LIMIT {
            // self.show();
            let prev_is_whitespace = self.current_char().is_whitespace();
            self.right(1);
            let curr_is_word = !self.current_char().is_whitespace();

            // println!(
            //     "curr '{}', prev '{}', cw={}, pw={}, wv={} index={}",
            //     self.current_char(),
            //     self.prev_char().unwrap_or(' '),
            //     curr_is_word,
            //     prev_is_whitespace,
            //     words_visited,
            //     self.c,
            // );

            if curr_is_word && prev_is_whitespace {
                words_visited += 1;
            }
            if words_visited == count {
                return;
            }
        }

        warn!("hit iteration limit in advance_word - probably a bug");
    }

    pub fn e(&mut self, count: usize) {
        for _ in 0..count {
            loop {
                self.right(1);
                let c = self.current_char();
                let d = self.next_char().unwrap_or(' ');

                let at_boundary = !c.is_whitespace() && d.is_whitespace();
                if at_boundary {
                    break;
                }
            }
        }
    }

    /// vim-motion, go back by word
    pub fn b(&mut self, count: usize) {
        // if start of word, then actually go to first char of next word,
        // otherwise it's start of word
        if count == 0 {
            return;
        }

        // Move at least one letter left

        for i in 0..50 {
            let right = self.current_char();
            self.left(1);
            let left = self.current_char();
            // but we are on the left now! Need to ... mvoe back?
            let is_boundary = left.is_whitespace() && !right.is_whitespace() && i > 0;

            // println!(
            //     "left='{}', right='{}', index={}, boundary={}, count={}",
            //     left, right, self.c, is_boundary, count
            // );

            if is_boundary {
                self.right(1); // I hate this
                if count > 0 {
                    return self.b(count - 1);
                }
                return;
            }
        }
        warn!("prev_word: hit iteration limit, probably at start of file");
    }

    pub fn clear_target_col(&mut self) {
        self.target_col = None;
    }

    pub fn j(&mut self, count: usize) {
        if self.target_col.is_none() {
            self.target_col = Some(self.col);
            info!("setting target_col to {}", self.col);
        }

        for _ in 0..count {
            if self.row == self.num_lines() - 1 {
                return;
            }

            // let's move forward until we meet a new line. After that we'll move
            // forward until we either reach target_col, or we reach another newline

            while self.current_char() != '\n' {
                self.right(1);
            }

            for _ in 0..self.target_col.unwrap() + 1 {
                self.right(1);
                if self.next_char().map(|c| c == '\n').unwrap_or(true) {
                    break;
                }
            }
        }
    }

    pub fn k(&mut self, count: usize) {
        if self.target_col.is_none() {
            self.target_col = Some(self.col);
            info!("setting target_col to {}", self.col);
        }

        for _ in 0..count {
            // let's move backward until we meet a new line. After that we'll move
            // backward until we either reach another newline or start of buffer

            if self.row == 0 {
                return;
            }

            let mut maxiter = 0;

            while self.current_char() != '\n' {
                self.left(1);
                maxiter += 1;
                if maxiter > 1000 {
                    panic!("hit iteration limit in k - probably a bug");
                }
            }

            // now let's move to start of line...
            self.left(1);
            while self.col > 0 {
                self.left(1);
                maxiter += 1;
                if maxiter > 1000 {
                    panic!("hit iteration limit in k - probably a bug");
                }
            }

            // ... and let's now move forward, like in the j case
            for _ in 0..self.target_col.unwrap() {
                if self.next_char().map(|c| c == '\n').unwrap_or(true) {
                    break;
                }
                self.right(1);
            }
        }
    }

    pub fn current_line(&self) -> String {
        self.text().lines().nth(self.row).unwrap_or("").to_string()
    }

    fn num_lines(&self) -> usize {
        self.text().lines().count()
    }

    fn prev_char(&self) -> Option<char> {
        if self.c == 0 {
            None
        } else {
            Some(self.buf[self.c - 1])
        }
    }

    /// returns the char the cursor is currently located at
    fn current_char(&self) -> char {
        // How the hell do I know if I read from c or d?
        self.buf[self.d]
    }
    fn next_char(&self) -> Option<char> {
        if self.d >= self.buf.len() - 1 {
            None
        } else {
            Some(self.buf[self.d + 1])
        }
    }

    #[allow(dead_code)]
    fn show(&self) -> String {
        let s: String = self
            .buf
            .iter()
            .map(|&c| if c == '\n' { 'N' } else { c })
            .collect();
        let mut v2 = vec!['.'; self.buf.capacity() + 1];
        v2[self.c] = 'C';
        v2[self.d] = 'D';
        let s2: String = v2.iter().collect();
        let s3 = format!("\n{}\n{}", s, s2);
        println!("{}", s3);
        s3
    }

    pub fn eol(&mut self) {
        info!("next is eol: {}", self.next_char().is_none());
        const LIMIT: usize = 150;
        for _ in 0..LIMIT {
            match self.next_char() {
                None | Some('\n') => {
                    info!("at eol, current char '{}'", self.current_char());
                    self.target_col = Some(9999); // always be eol'ing
                    return;
                }
                Some(c) => {
                    info!(
                        "not at eol, met '{}' and curr is {}",
                        c,
                        self.current_char()
                    );
                    self.right(1);
                }
            }
        }
        warn!("hit iteration limit in eol - probably a bug");
    }
}

// sequence of letters, digits, underscores, or a sequence of other
// non-blank characters.
fn is_word(c: char) -> bool {
    c.is_alphanumeric() || "[]().,$_".chars().any(|h| h == c)
}
fn is_WORD(c: char) -> bool {
    is_word(c) // TODO
}
fn is_blank(c: char) -> bool {
    c.is_whitespace() || c == '\n'
}

impl Iterator for Buffer {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        if self.d < self.buf.len() {
            let c = self.buf[self.d];
            self.right(1);
            Some(c)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
