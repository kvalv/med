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

use log::{info, log, warn};

use crate::{
    buffer::history::{Change, ChangeHistory, Operation},
    cmd::pattern::Motion,
    span::{Position, Span},
    textobject::{Boundary, TextObject},
};
pub mod history;

/// A Buffer represents a file that is being edited.
/// Implemented as a gap buffer
#[derive(Debug, Clone)]
pub struct Buffer {
    buf: Vec<char>,
    c: usize,
    d: usize,

    pub row: usize,
    pub col: usize,

    // when moving up or donw, the target column is what the desired column is to be at given
    // enough caracters, when doing jjjj or kkkk
    target_col: Option<usize>,

    /// Undo and redo
    changes: ChangeHistory,
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
            changes: ChangeHistory::default(),
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

    /// Returns true if the cursor is at the end of the file. Current_char
    /// points to the last char.
    pub fn at_eof(&self) -> bool {
        self.d >= self.buf.len() - 1
    }

    fn back_while<T>(&self, start: usize, cond: T) -> usize
    where
        T: Fn(Option<char>, char) -> bool,
    {
        const LIMIT: usize = 10000;
        for i in 0..LIMIT {
            if start - i == 0 {
                return i;
            }
            let curr = self.buf[start - i];
            let prev = if start - i == 0 {
                None
            } else {
                Some(self.buf[start - i - 1])
            };
            let got = cond(prev, curr);
            // println!(
            //     "go back while: prev={:?} curr={:?} continue={}",
            //     prev, curr, got
            // );
            if !got {
                return i;
            }
        }
        panic!("hit iteration limit in go_back_while - probably a bug");
    }

    /// (current, next) -> bool. Returns the offset
    fn forward_while<T>(&self, start: usize, cond: T) -> usize
    where
        T: Fn(char, Option<char>) -> bool,
    {
        const LIMIT: usize = 10000;
        for i in 0..LIMIT {
            if start + i >= self.buf.len() {
                return i;
            }
            let curr = self.buf[start + i];
            let next = if start + i + 1 >= self.buf.len() {
                None
            } else {
                Some(self.buf[start + i + 1])
            };
            if !cond(curr, next) {
                return i;
            }
        }
        panic!("hit iteration limit in advance_while - probably a bug");
    }

    pub fn d(&mut self, count: usize, boundary: Boundary, obj: TextObject) {
        use Boundary::*;
        use TextObject::*;

        match (boundary, obj) {
            (Current, Word) => {
                for _ in 0..count {
                    if self.d >= self.buf.len() {
                        if self.c > 0 {
                            self.c -= 1;
                        }
                        return;
                    }
                    let start = self.current_char();

                    if start.is_whitespace() {
                        // handle that by just munching whitespace
                        let i = self
                            .forward_while(self.d, |_, next| next.map(is_word).unwrap_or(false));
                        self.d += i;
                        return;
                    }

                    // otherwise it's a word. We'll eat up all words,
                    // and then we'll eat up any trailing whitespace

                    let i = self.forward_while(self.d, |curr, _| is_word(curr));

                    self.d += i;
                    let j = self.forward_while(self.d, |curr, _| curr == ' ');
                    self.d += j;
                }
            }
            (Inner, Word) => {
                // if we do a 'diw' on whitespace --> the whitespace should be
                // killed. Otherwise, the word should be killed

                // a predicate deciding what we want to remove
                let want_removed = if self.current_char().is_whitespace() {
                    is_blank
                } else {
                    is_word
                };

                let i = self.back_while(self.c, |prev, _| prev.map(want_removed).unwrap_or(false));
                self.c -= i;
                self.col -= i;

                // forward
                let j =
                    self.forward_while(self.d, |_, next| next.map(want_removed).unwrap_or(false));
                // let end = self.advance_while(self.d, is_word);
                self.d += j + 1;
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
            if self.c == 0 || self.col == 0 {
                return;
            }
            self.c -= 1;
            self.col -= 1;
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

    pub fn undo(&mut self) {
        if let Some(change) = self.changes.undo() {
            // Undo means we want to use the 'old'
            // Delete the 'new text' at a given span, then insert

            // self.delete_span(change.span, true);

            self.insert_text(change.span.start, &change.old);
            // println!(
            //     "insert text {} chars '{}' at pos {} - new text is '{}'",
            //     change.old.len(),
            //     &change.old,
            //     change.span.start,
            //     self.text(),
            // );
        }
    }
    pub fn redo(&mut self) {
        if let Some(change) = self.changes.redo() {
            todo!();
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
    pub fn update_target_col(&mut self) {
        self.target_col = Some(self.col);
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

    // fn prev_char(&self) -> Option<char> {
    //     if self.c == 0 {
    //         None
    //     } else {
    //         Some(self.buf[self.c - 1])
    //     }
    // }

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

    /// current position of the cursor
    pub fn current_position(&self) -> Position {
        let mut pos = Position { row: 0, col: 0 };
        for c in self.buf.iter().take(self.c) {
            if *c == '\n' {
                pos.row += 1;
                pos.col = 0;
            } else {
                pos.col += 1;
            }
        }
        pos
    }

    pub fn register_change(&mut self, change: Change) {
        info!("Change registered: ");
        self.changes.register(change);
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

    fn num_columns(&self, row: usize) -> usize {
        match self.text().lines().nth(row) {
            None => 0,
            Some(line) => line.char_indices().count(),
        }
    }

    pub fn span(&self, motion: Motion) -> Span {
        let wc = WordClass::from(self.current_char());
        let count = motion.count.unwrap_or(1);

        // let last_iteration = |k: usize| k == count - 1;
        // let at_boundary = |idx: usize| idx == 0 || (idx >= self.buf.len() - 1);
        // let char_at = |idx: usize| {
        //     if idx >= self.buf.len() {
        //         '\0'
        //     } else {
        //         self.buf[idx]
        //     }
        // };

        let mut back: usize = 0;
        let mut fwd: usize = 0;
        let mut empty_span = false;

        use Boundary::*;
        use TextObject::*;

        match (motion.boundary, motion.object) {
            (Around, Paren) | (Around, CurlyBracket) => {
                (back, fwd, empty_span) = self.span_around_symbol(
                    motion.object.open_symbol().unwrap(),
                    motion.object.close_symbol().unwrap(),
                    count,
                );
            }
            (Inner, Paren) | (Inner, CurlyBracket) => {
                let span = self.span(Motion {
                    boundary: Around,
                    object: motion.object,
                    count: Some(count),
                });
                return if span.is_empty() {
                    span
                } else {
                    span.shrink(1)
                };
            }
            (Current, Word) => {
                for _ in 0..count {
                    // munch characters until end of wordclass
                    fwd += self
                        .forward_while(self.d, |_, next| next.map(|c| wc.eq(c)).unwrap_or(false));
                    // next pass: munch whitespaces
                    fwd += self.forward_while(self.d + fwd + 1, |curr, _| {
                        WordClass::from(curr) == WordClass::Whitespace
                    });
                    // finally: if there's more text, let's put cursor on the
                    // next char
                    if self.d + fwd + 1 < self.buf.len() {
                        fwd += 1;
                    }
                }
            }
            (Inner, Word) => {
                // backwards pass
                back = self.back_while(self.c, |prev, _| prev.map(|c| wc.eq(c)).unwrap_or(false));
                // println!("would go back by {} chars", back);

                // forward
                fwd = 0;
                let mut wc_forward = wc;
                for k in 0..count {
                    // println!(
                    //     "k={} Start char is '{}' and wc is {:?} and index is {}",
                    //     k,
                    //     self.buf[self.d + fwd],
                    //     wc_forward,
                    //     self.d + fwd
                    // );
                    let tmp = self.forward_while(self.d + fwd, |curr, next| {
                        // println!("curr='{}' next='{:?}'", curr, next);
                        next.map(|c| wc_forward.eq(c)).unwrap_or(false)
                    });
                    fwd += tmp;

                    if k == count - 1 {
                        break;
                    }

                    // if already at end -> nothing to do
                    if self.d + fwd + 1 >= self.buf.len() {
                        // println!("At end nothing to do");
                        break;
                    }

                    fwd += 1; // 

                    // println!(
                    //     "buf.len={} d={} j={} tmp={} d will look at '{}'",
                    //     self.buf.len(),
                    //     self.d,
                    //     fwd,
                    //     tmp,
                    //     if self.d + fwd >= self.buf.len() {
                    //         'X'
                    //     } else {
                    //         self.buf[self.d + fwd]
                    //     }
                    // );

                    // // if more -> advance cursor one more step??
                    // // and now we want
                    wc_forward = if self.d + fwd >= self.buf.len() {
                        WordClass::Whitespace
                    } else {
                        WordClass::from(self.buf[self.d + fwd])
                    };
                    // println!("self.d+j = {}", self.d + fwd);
                    // println!(
                    //     "Moved forward by {}, char is '{}' and wc_forward is {:?}",
                    //     tmp,
                    //     self.buf[self.d + j],
                    //     wc_forward
                    // );
                }
                //
            }
            _ => todo!(),
        }

        // println!("go back by {} and forward by {}", back, fwd);

        if empty_span {
            // e.g. due to '2a(' when there is no second paranthesis
            // --> empty span (no words selected)
            return Span::empty_at(self.row, self.col);
        }

        let mut span = Span {
            start: Position {
                row: self.row,
                col: self.col,
            },
            end: Position {
                row: self.row,
                col: self.col + 1,
            },
        };

        for k in 0..back {
            if self.buf[self.c - k] == '\n' {
                span.start.row -= 1;
                span.start.col = self.num_columns(span.start.row);
                // println!(
                //     "subtracting one in span.start.row and setting col to {}",
                //     span.start.col
                // );
            } else {
                span.start.col -= 1;
                // println!("subtracting one in span.start.col");
            }
        }
        for k in 0..fwd {
            if self.buf[self.d + k] == '\n' {
                span.end.row += 1;
                span.end.col = 1; // To include? I added col=1 to make va( with
            // newline work.
            } else {
                span.end.col += 1;
            }
        }
        // println!(
        //     "resulting span is ({}, {}) to ({}, {})",
        //     span.start.row, span.start.col, span.end.row, span.end.col
        // );
        span
    }

    pub fn delete_span(&mut self, span: Span, include_end: bool) -> String {
        self.position(span.start.row, span.start.col);

        let mut loc = Position { row: 0, col: 0 };
        let mut end_index = self
            .text()
            .char_indices()
            .find(|(_, c)| {
                if loc == span.end || loc.row > span.end.row {
                    return true;
                }
                if *c == '\n' {
                    loc.row += 1;
                    loc.col = 1; // To include? I added col=1 to make va( with
                } else {
                    loc.col += 1;
                }
                false
            })
            .map(|(i, _)| i);

        if include_end {
            end_index = end_index.map(|v| v + 1);
        }

        if let Some(end_index) = end_index {
            let diff = end_index - self.c;
            let deleted_text: String = self.buf.iter().skip(self.d).take(diff).collect();
            self.d += diff;
            deleted_text
        } else {
            panic!("end_index");
        }
    }

    /// Inserts text at a given position, preserving the cursor location
    pub fn insert_text(&mut self, insert_at: Position, text: &str) {
        let pos = self.current_position();

        self.position(insert_at.row, insert_at.col);
        for c in text.chars() {
            self.insert(c);
        }

        let end_pos = if pos < insert_at {
            // inserted text is to the right of the cursor, so the cursor stays
            insert_at
        } else {
            // to the left -> we need to adjust. We'll add the implied from the text
            pos + Span::from(text)
        };
        self.position(end_pos.row, end_pos.col);
    }

    // A helper function for span. The logic for a(, a{, a[, ... is the same, just with different symbols.
    fn span_around_symbol(
        &self,
        open_symbol: char,
        close_symbol: char,
        count: usize,
    ) -> (usize, usize, bool) {
        let mut back: usize = 0;
        let mut fwd: usize = 0;
        let mut empty_span = false;

        let char_at = |idx: usize| {
            if idx >= self.buf.len() {
                '\0'
            } else {
                self.buf[idx]
            }
        };
        let at_boundary = |idx: usize| idx == 0 || (idx >= self.buf.len() - 1);
        let last_iteration = |k: usize| k == count - 1;

        for k in 0..count {
            back += self.back_while(self.c - back, |_, curr| curr != open_symbol);
            fwd += self.forward_while(self.d + fwd, |curr, _| curr != close_symbol);
            // println!("k={} back={} fwd={}", k, back, fwd);
            if !last_iteration(k) && !at_boundary(self.c - back) {
                back += 1;
            }
            if !last_iteration(k) && !at_boundary(self.d + fwd) {
                fwd += 1;
            }
        }

        if !(char_at(self.c - back) == open_symbol && char_at(self.d + fwd) == close_symbol) {
            empty_span = true;
        }

        (back, fwd, empty_span)
    }

    pub fn text_for_span(&self, span: Span) -> String {
        // panic!("stop here for now");
        // TODO: not this
        let mut cloned = self.clone();
        let mut chars = vec![];
        cloned.position(span.start.row, span.start.col);
        const LIMIT: usize = 10000;
        for _ in 0..LIMIT {
            if cloned.row == span.end.row && cloned.col == span.end.col {
                return chars.iter().collect();
            }
            chars.push(cloned.current_char());
            if cloned.at_eof() {
                return chars.iter().collect();
            }
            cloned.right(1);
        }
        panic!("hit iteration limit in text_for_span - probably a bug");
    }
}

// sequence of letters, digits, underscores, or a sequence of other
// non-blank characters.
fn is_word(c: char) -> bool {
    c.is_alphanumeric() || "[]().,$_".chars().any(|h| h == c)
}
// fn is_WORD(c: char) -> bool {
//     is_word(c) // TODO
// }
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

#[derive(Debug, PartialEq, Eq, Clone)]
enum WordClass {
    AlphaNumeric, // digit, alphabet, underscore
    Whitespace,   // space or tab
    Symbols,      // anything except the others here
    Newline,
}

impl WordClass {
    /// Returns true if the rhs char is the same wordclass as the current
    fn eq<T>(&self, rhs: T) -> bool
    where
        T: Into<WordClass>,
    {
        rhs.into() == *self
    }
}

impl From<char> for WordClass {
    fn from(value: char) -> Self {
        if value.is_alphanumeric() {
            Self::AlphaNumeric
        } else if value == '\n' {
            Self::Newline
        } else if value == ' ' || value == '\t' {
            Self::Whitespace
        } else {
            Self::Symbols
        }
    }
}

#[cfg(test)]
mod tests;
