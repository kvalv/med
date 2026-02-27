// Gap buffer:
// B, C, CE and E
// prefix: B -> C
// suffic: CE -> E
//
// moving left -> keep gap size constant -> has to shift one element to the right set
// moving right: similar as above
// write: just insert, unless need to grow. if need grow -> grow, then insert
// remove? move pointer to the left.

use log::{info, warn};

/// A Buffer represents a file that is being edited.
/// Implemented as a gap buffer
#[derive(Debug)]
pub struct Buffer {
    buf: Vec<char>,
    c: usize,
    d: usize,

    pub row: usize,
    pub col: usize,
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
        buffer
    }
}

impl Buffer {
    fn new(cap: usize) -> Self {
        Self {
            row: 0,
            col: 0,
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
    fn left(&mut self, n: usize) {
        for _ in 0..n {
            if self.c == 0 {
                return;
            }

            // d receives content from left side, then decrement
            let recv = self.buf[self.c - 1];

            self.buf[self.d - 1] = recv;

            if recv == '\n' {
                self.row = self.row.saturating_sub(1);
                // we need to recalculate col by counting chars until the next newline
                self.col = self.text().chars().rev().take_while(|c| *c != '\n').count();
            } else {
                self.col = self.col.saturating_sub(1);
            }

            // Then shift pointers
            self.c -= 1;
            self.d -= 1;
        }
    }
    fn right(&mut self, n: usize) {
        for _ in 0..n {
            if self.d == self.buf.len() {
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

    /// Moves to a particular line
    pub fn position(&mut self, row: usize, col: usize) {
        let mut r = 0;
        let last_row = self.text().chars().filter(|c| *c == '\n').count() - 1;
        let num_rows = last_row + 1;

        for (i, c) in self.text().chars().enumerate() {
            if c == '\n' {
                r += 1;
            }
            if row == r || r == last_row {
                // we found the row!
                // let dst = i + col + 1;
                // let mut dst = i;
                let col2 = if r == last_row && row > num_rows {
                    999
                } else {
                    col
                };

                let j = self
                    .text()
                    .chars()
                    .skip(i + 1)
                    .take_while(|c| *c != '\n')
                    .count()
                    .saturating_sub(1)
                    .min(col2);
                let dst = i + j + 1;

                // println!(
                //     "found the line row={} dst={} cursor at {} - going {} {} steps",
                //     row,
                //     dst,
                //     self.c,
                //     if dst > self.c { "left" } else { "right" },
                //     (dst as i32 - self.c as i32).abs(),
                // );

                if dst > self.c {
                    self.right(dst - self.c);
                } else {
                    self.left(self.c - dst);
                }

                self.row = row;
                self.col = j;

                return;
            }
        }
    }
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

    /// Returns the number of characters advanced
    pub fn advance_word(&mut self, count: isize) {
        if count < 0 {
            self.prev_word((-count) as usize);
            return;
        }
        if count == 0 {
            info!(
                "count is 0, doing nothing - current char '{}' at index {}",
                self.current_char(),
                self.c,
            );
        }

        let mut words_visited = 0;

        for _ in 0..10_000 {
            self.right(1);
            let curr_is_word = !self.current_char().is_whitespace();
            let prev_is_whitespace = self.prev_char().map(|x| x.is_whitespace()).unwrap_or(false);

            if curr_is_word && prev_is_whitespace {
                words_visited += 1;
            }
            if words_visited == count {
                return;
            }
        }

        warn!("hit iteration limit in advance_word - probably a bug");
    }

    fn prev_word(&mut self, count: usize) {
        // if start of word, then actually go to first char of next word,
        // otherwise it's start of word
        if count == 0 {
            return;
        }

        // Move at least one letter left
        self.left(1);

        for _ in 0..50 {
            let curr_is_word = !self.current_char().is_whitespace();
            let prev_is_whitespace = self.prev_char().map(|x| x.is_whitespace()).unwrap_or(false);

            let is_boundary = curr_is_word && prev_is_whitespace;
            if is_boundary {
                if count > 0 {
                    return self.prev_word(count - 1);
                }
                return;
            }
            self.left(1);
        }
        warn!("prev_word: hit iteration limit, probably at start of file");
    }

    pub fn current_line(&self) -> String {
        self.text().lines().nth(self.row).unwrap_or("").to_string()
    }

    /// returns the char the cursor is currently located at
    fn current_char(&self) -> char {
        self.buf[self.c]
    }
    fn prev_char(&self) -> Option<char> {
        if self.c == 0 {
            None
        } else {
            Some(self.buf[self.c - 1])
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
        format!("{}\n{}", s, s2)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        let mut g = Buffer::new(10);
        g.insert('h');
        g.insert('i');
        assert_eq!(g.text(), "hi");
    }

    #[test]
    fn left() {
        let mut g = Buffer::new(10);
        g.insert('h');
        g.insert('i');
        g.left(1);
        g.insert('a');
        g.show();
        assert_eq!(g.text(), "hai");
    }

    #[test]
    fn backspace() {
        let mut g = Buffer::new(10);
        g.insert('h');
        g.insert('i');
        g.left(1);
        g.backspace(1);
        g.show();
        assert_eq!(g.text(), "i");
    }

    #[test]
    fn grow() {
        let mut g = Buffer::new(3);
        g.insert('1');
        g.insert('2');
        g.insert('3');
        g.insert('4');
        assert_eq!(g.text(), "1234", "Failed:\n{}\n", g.show());
    }

    #[test]
    fn more_cases() {
        let cases: Vec<(&str, &str)> = vec![
            ("12X3", "13"),
            ("11LL22LL33", "332211"),
            ("1L2RR3", "213"),
            ("1LR2", "12"),
        ];

        for (i, (seq, want)) in cases.into_iter().enumerate() {
            let mut g = Buffer::new(10);
            for cmd in seq.chars() {
                match cmd {
                    'X' => g.backspace(1),
                    'L' => g.left(1),
                    'R' => g.right(1),
                    '0'..='9' => g.insert(cmd),
                    _ => panic!("unknown character '{}'", cmd),
                }
            }
            assert_eq!(want, g.text(), "Case {} failed: \n{}", i, g.show());
        }
    }

    #[test]
    fn text_line_count() {
        let g = Buffer::from("the cat\nsat in\na tree");
        assert_eq!(g.text().lines().count(), 3);
    }

    #[test]
    fn newlines() {
        let mut g = Buffer::from("the cat\nsat in\na tree");
        assert_eq!("the cat\nsat in\na tree", g.text());

        g.position(2, 2); // 'a t' -> t
        assert_eq!('t', g.current_char());

        assert_eq!(2, g.row);
        assert_eq!(2, g.col);

        g.position(0, 100);
        assert_eq!('t', g.current_char());
        assert_eq!(0, g.row);
        // assert_eq!(6, g.col); // TODO

        g.position(1, 0);
        assert_eq!('s', g.current_char());
        assert_eq!("the cat\nsat in\na tree", g.text());
        assert_eq!(1, g.row);
        assert_eq!(0, g.col);

        // TODO: this ain't working yet
        // g.position(5, 0); // last line, last char
        // println!("{}", g.show());
        // assert_eq!('e', g.current_char());
    }

    #[test]
    fn test_advance_word_forward() {
        struct Testcase {
            input: &'static str,
            count: isize,
            want_char: char,
        }

        let cases = [
            Testcase {
                input: "The cat sat",
                count: 1,
                want_char: 'c',
            },
            Testcase {
                input: "The cat sat",
                count: 2,
                want_char: 's',
            },
            // Testcase {
            //     input: "the Cat sat",
            //     count: 1,
            //     want_char: 's',
            // }, // Testcase {
            //     input: "tHe cat sat",
            //     count: 1,
            //     want_char: 'c',
            // },
            // Testcase {
            //     input: "a B ] long",
            //     count: 1,
            //     want_char: ']',
            // },
        ];

        // let _ = env_logger::builder().is_test(true).try_init();
        for (i, tc) in cases.iter().enumerate() {
            let start = tc
                .input
                .chars()
                .position(|c| c.is_ascii_uppercase())
                .unwrap();
            let text = tc.input.to_lowercase();
            let len = text.len();
            let mut b = Buffer::from(text.as_str());
            b.left(len - start);
            println!("start\n{}\n", b.show());
            b.advance_word(tc.count);
            println!("after\n{}\n", b.show());
            assert_eq!(tc.want_char, b.current_char(), "Case {i} failed");
        }
    }

    #[test]
    fn test_advance_in_readme() {
        let readme = std::fs::read_to_string("README.md").unwrap();
        let mut b = Buffer::from(readme.as_str());
        b.position(0, 0);
        b.advance_word(2);
        assert_eq!(']', b.current_char());
    }

    #[test]
    fn test_advance_word_backward() {
        struct Testcase {
            input: &'static str,
            count: isize,
            want_char: char,
        }

        let cases = [
            Testcase {
                input: "the cat Sat",
                count: -1,
                want_char: 'c',
            },
            Testcase {
                input: "the cat sAt",
                count: -1,
                want_char: 's',
            },
            Testcase {
                input: "the cat Sat",
                count: -2,
                want_char: 't',
            },
            Testcase {
                input: "the cat sAt",
                count: -2,
                want_char: 'c',
            },
            Testcase {
                input: "the cat    sAt",
                count: -2,
                want_char: 'c',
            },
        ];

        for (i, tc) in cases.iter().enumerate() {
            let start = tc
                .input
                .chars()
                .position(|c| c.is_ascii_uppercase())
                .unwrap();
            let text = tc.input.to_lowercase();
            let len = text.len();
            let mut b = Buffer::from(text.as_str());
            b.left(len - start);
            b.advance_word(tc.count);
            assert_eq!(tc.want_char, b.current_char(), "Case {i} failed");
        }
    }
}
