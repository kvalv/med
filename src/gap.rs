// Gap buffer:
// B, C, CE and E
// prefix: B -> C
// suffic: CE -> E
//
// moving left -> keep gap size constant -> has to shift one element to the right set
// moving right: similar as above
// write: just insert, unless need to grow. if need grow -> grow, then insert
// remove? move pointer to the left.
struct Gap {
    buf: Vec<char>,
    c: usize,
    d: usize,
}

impl Gap {
    fn new(cap: usize) -> Self {
        Self {
            buf: vec!['x'; cap],
            c: 0,
            d: cap,
        }
    }
    fn text(&self) -> String {
        self.buf
            .iter()
            .take(self.c)
            .chain(self.buf.iter().skip(self.d))
            .collect()
    }
    fn left(&mut self) {
        if self.c == 0 {
            return;
        }
        // Shift over content to right side
        self.buf[self.d - 1] = self.buf[self.c - 1];

        // Then shift pointers
        self.c -= 1;
        self.d -= 1;
    }
    fn right(&mut self) {
        if self.d == self.buf.len() {
            return;
        }
        self.buf[self.c] = self.buf[self.d];

        // Then shift pointers
        self.d += 1;
        self.c += 1;
    }
    fn insert(&mut self, v: char) {
        if self.c == self.d {
            self.grow();
        }
        self.buf[self.c] = v;
        self.c += 1;
    }
    fn backspace(&mut self) {
        if self.c == 0 {
            return;
        }
        self.c -= 1;
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

    fn show(&self) -> String {
        let s: String = self.buf.iter().collect();
        let mut v2 = vec!['.'; self.buf.len()];
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
        let mut g = Gap::new(10);
        g.insert('h');
        g.insert('i');
        assert_eq!(g.text(), "hi");
    }

    #[test]
    fn left() {
        let mut g = Gap::new(10);
        g.insert('h');
        g.insert('i');
        g.left();
        g.insert('a');
        g.show();
        assert_eq!(g.text(), "hai");
    }

    #[test]
    fn backspace() {
        let mut g = Gap::new(10);
        g.insert('h');
        g.insert('i');
        g.left();
        g.backspace();
        g.show();
        assert_eq!(g.text(), "i");
    }

    #[test]
    fn grow() {
        let mut g = Gap::new(3);
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
            let mut g = Gap::new(10);
            for cmd in seq.chars() {
                match cmd {
                    'X' => g.backspace(),
                    'L' => g.left(),
                    'R' => g.right(),
                    '0'..='9' => g.insert(cmd),
                    _ => panic!("unknown character '{}'", cmd),
                }
            }
            assert_eq!(want, g.text(), "Case {} failed: \n{}", i, g.show());
        }
    }
}
