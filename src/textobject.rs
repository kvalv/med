use std::ops::Index;

use crate::{buffer::Buffer, span::Span};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextObject(pub Variant, pub Object);

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Copy)]
pub enum Object {
    #[default]
    Paren, // ()
    CurlyBracket, // {}
    Word,
    WordEnd, // e
    // Paren, // ()
    // Brack, // <>
    Back, // b
}

impl Object {
    pub fn open_symbol(&self) -> Option<char> {
        match self {
            Object::Paren => Some('('),
            Object::CurlyBracket => Some('{'),
            _ => None,
        }
    }
    pub fn close_symbol(&self) -> Option<char> {
        match self {
            Object::Paren => Some(')'),
            Object::CurlyBracket => Some('}'),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Copy)]
pub enum Variant {
    #[default]
    Inner,
    // includes whitespace . Prefers whitespace ahead. If not possible (eg due to different
    // word type) then uses the other side.
    // Not really clear to me the logic of how this is in vim...
    Around,
}

/// If input contains digits at the start, they are parsed as an usize. The returned slice is the
/// input slice stripped.
fn extract_count(input: &str) -> (Option<usize>, &str) {
    let count: String = input.chars().take_while(|c| c.is_numeric()).collect();

    if count.is_empty() {
        (None, input)
    } else {
        (count.parse::<usize>().ok(), input.index(count.len()..))
    }
}

// textobject: a|i <object>

pub fn parse_textobject(input: &str) -> (Option<(Object, Variant, Option<usize>)>, usize) {
    let (count, rest) = extract_count(input);
    let mut consumed = count.iter().len();

    let mut boundary = Variant::Inner;
    let mut object: Option<Object> = None;

    let mut it = rest.chars();
    match it.next() {
        Some('a') => boundary = Variant::Around,
        Some('i') => boundary = Variant::Inner,
        Some('w') => object = Some(Object::Word),
        Some('e') => object = Some(Object::WordEnd),
        Some('(') => object = Some(Object::Paren),
        Some('{') => object = Some(Object::CurlyBracket),
        _ => return (None, 0),
    }
    consumed += 1;

    match it.next() {
        Some('w') => object = Some(Object::Word),
        Some('e') => object = Some(Object::WordEnd),
        Some('(') => object = Some(Object::Paren),
        Some('{') => object = Some(Object::CurlyBracket),
        _ => {}
    };

    if let Some(object) = object {
        consumed += 1;
        return (Some((object, boundary, count)), consumed);
    }

    (None, 0)
}

impl TextObject {
    pub fn span(&self, buf: &Buffer) -> Span {
        todo!();

        // let wc = WordClass::from(self.current_char());
        // let obj = text_object.object;
        // let boundary = text_object.variant;

        // // let last_iteration = |k: usize| k == count - 1;
        // // let at_boundary = |idx: usize| idx == 0 || (idx >= self.buf.len() - 1);
        // // let char_at = |idx: usize| {
        // //     if idx >= self.buf.len() {
        // //         '\0'
        // //     } else {
        // //         self.buf[idx]
        // //     }
        // // };

        // let mut back: usize = 0;
        // let mut fwd: usize = 0;
        // let mut empty_span = false;

        // use Object::*;
        // use TextObjectVariant::*;
        // let text_object = TextObject {
        //     object: obj,
        //     variant: boundary,
        // };

        // match (boundary, obj) {
        //     (Around, Paren) | (Around, CurlyBracket) => {
        //         (back, fwd, empty_span) = self.span_around_symbol(
        //             obj.open_symbol().unwrap(),
        //             obj.close_symbol().unwrap(),
        //             count,
        //         );
        //     }
        //     (Inner, Paren) | (Inner, CurlyBracket) => {
        //         let span = self.span_for_textobject(obj, Around, count);
        //         return if span.is_empty() {
        //             span
        //         } else {
        //             span.shrink(1)
        //         };
        //     }
        //     (Inner, Word) => {
        //         // backwards pass
        //         back = self.back_while(self.c, |prev, _| {
        //             prev.map(|c| WordClass::from(c) == wc).unwrap_or(false)
        //         });
        //         // println!("would go back by {} chars", back);

        //         // forward
        //         fwd = 0;
        //         let mut wc_forward = wc;
        //         for k in 0..count {
        //             // println!(
        //             //     "k={} Start char is '{}' and wc is {:?} and index is {}",
        //             //     k,
        //             //     self.buf[self.d + fwd],
        //             //     wc_forward,
        //             //     self.d + fwd
        //             // );
        //             let tmp = self.forward_while(self.d + fwd, |_, next| {
        //                 // println!("curr='{}' next='{:?}'", curr, next);
        //                 next.map(|c| WordClass::from(c) == wc_forward)
        //                     .unwrap_or(false)
        //             });
        //             fwd += tmp;

        //             if k == count - 1 {
        //                 break;
        //             }

        //             // if already at end -> nothing to do
        //             if self.d + fwd + 1 >= self.buf.len() {
        //                 // println!("At end nothing to do");
        //                 break;
        //             }

        //             fwd += 1; //

        //             // println!(
        //             //     "buf.len={} d={} j={} tmp={} d will look at '{}'",
        //             //     self.buf.len(),
        //             //     self.d,
        //             //     fwd,
        //             //     tmp,
        //             //     if self.d + fwd >= self.buf.len() {
        //             //         'X'
        //             //     } else {
        //             //         self.buf[self.d + fwd]
        //             //     }
        //             // );

        //             // // if more -> advance cursor one more step??
        //             // // and now we want
        //             wc_forward = if self.d + fwd >= self.buf.len() {
        //                 WordClass::WHITESPACE
        //             } else {
        //                 WordClass::from(self.buf[self.d + fwd])
        //             };
        //             // println!("self.d+j = {}", self.d + fwd);
        //             // println!(
        //             //     "Moved forward by {}, char is '{}' and wc_forward is {:?}",
        //             //     tmp,
        //             //     self.buf[self.d + j],
        //             //     wc_forward
        //             // );
        //         }
        //         //
        //     }
        //     _ => todo!(),
        // }

        // // println!("go back by {} and forward by {}", back, fwd);

        // if empty_span {
        //     // e.g. due to '2a(' when there is no second paranthesis
        //     // --> empty span (no words selected)
        //     return Span::empty_at(self.row, self.col);
        // }

        // let mut span = Span {
        //     start: Position {
        //         row: self.row,
        //         col: self.col,
        //     },
        //     end: Position {
        //         row: self.row,
        //         col: self.col + 1,
        //     },
        // };

        // for k in 0..back {
        //     if self.buf[self.c - k] == '\n' {
        //         span.start.row -= 1;
        //         span.start.col = self.num_columns(span.start.row);
        //         // println!(
        //         //     "subtracting one in span.start.row and setting col to {}",
        //         //     span.start.col
        //         // );
        //     } else {
        //         span.start.col -= 1;
        //         // println!("subtracting one in span.start.col");
        //     }
        // }
        // for k in 0..fwd {
        //     if self.buf[self.d + k] == '\n' {
        //         span.end.row += 1;
        //         span.end.col = 1; // To include? I added col=1 to make va( with
        //     // newline work.
        //     } else {
        //         span.end.col += 1;
        //     }
        // }
        // // println!(
        // //     "resulting span is ({}, {}) to ({}, {})",
        // //     span.start.row, span.start.col, span.end.row, span.end.col
        // // );
        // span
    }
}

/*
    b = Buffer::from("hi\nworld");
    b.position(0, 0);
    b.d(1, Current, Word);
    assert_eq!("\nworld", b.text());
    // return;

    b = Buffer::from("the cat    sat");
    b.position(0, 5);
    b.d(1, Current, Word);
    assert_eq!("the csat", b.text());
    assert_eq!(b.row, 0);
    assert_eq!(b.col, 5);

    b = Buffer::from("hi");
    b.position(0, 0);
    b.d(1, Current, Word);
    assert_eq!("", b.text());

    b = Buffer::from("the cdog");
    b.position(0, 5);
    b.d(2, Current, Word);
    assert_eq!("the ", b.text());

    // when delete and reach end of word, also delete the current letter??? wtf?
    b = Buffer::from("the cat sat");
    b.position(0, 5);
    b.d(3, Current, Word);
    assert_eq!("the ", b.text());

    b = Buffer::from("- [x] navigation");
    b.position(0, 2);
    assert_eq!('[', b.current_char());
    b.d(1, Current, Word);
    assert_eq!("- navigation", b.text());



*/

#[cfg(test)]
mod tests {
    use super::*;
    use Object::*;
    use Variant::*;

    #[test]
    fn test_span() {
        let t = |input: &str, pos: (usize, usize), textobj: TextObject, want: &str| {
            let mut b = Buffer::from(input);
            b.position(pos.0, pos.1);
            let span = textobj.span(&b);
            let got = b.text_for_span(span);
            assert_eq!(
                want, got,
                "\ninput={input:?} pos={pos:?} textobj={textobj:?}"
            );
        };

        // around/inner paren
        t("(wow\n)", (0, 1), TextObject(Around, Paren), "(wow\n)");
        t("x(foo)y", (0, 1), TextObject(Inner, Paren), "foo");
        t("a (xxx) b", (0, 4), TextObject(Around, Paren), "(xxx)");
        t("a (inner) b", (0, 5), TextObject(Inner, Paren), "inner");
        // no match -> empty
        t(")x(", (0, 1), TextObject(Around, Paren), "");

        // around/inner curly bracket
        t("x{foo}y", (0, 1), TextObject(Around, CurlyBracket), "{foo}");
        t("x{foo}y", (0, 1), TextObject(Inner, CurlyBracket), "foo");

        // inner word
        t("the cat sat", (0, 5), TextObject(Inner, Word), "cat");
        t("abba", (0, 1), TextObject(Inner, Word), "abba");

        // punctuation word classes
        t("a.b", (0, 0), TextObject(Inner, Word), "a");
        t("a.b", (0, 1), TextObject(Inner, Word), ".");
        t("a..b", (0, 1), TextObject(Inner, Word), "..");
        t("a.!b", (0, 1), TextObject(Inner, Word), ".!");
        t("a.!b!", (0, 2), TextObject(Inner, Word), ".!");
    }
}
