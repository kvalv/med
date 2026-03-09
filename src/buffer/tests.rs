use crate::textobject::Object::*;
use crate::textobject::TextObject;
use crate::textobject::Variant::*;

use super::*;

// #[cfg(test)]
// mod tests {

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
        // ("1L2RR3", "213"),
        // ("1LR2", "12"),
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
}

#[test]
fn test_position() {
    let mut g = Buffer::from("test");
    g.position(0, 0);
    assert_eq!('t', g.current_char());
}

#[test]
fn test_position_2() {
    let mut g = Buffer::from("foo\nbar");
    g.position(0, 0);
    g.w(1);
    assert_eq!('b', g.current_char());
    g.b(1);
    assert_eq!('f', g.current_char());

    let mut g2 = Buffer::from("- [ ] insert some characters");
    g2.position(0, "- [ ] insert some ".len());
    g2.b(1);
    assert_eq!('s', g2.current_char());

    let mut g3 = Buffer::from("- [ ] open file\n- [ ] insert some characters\n- [x ] save file");
    g3.position(1, 10);

    println!("====");
    g3.show();
    println!("====");

    g3.b(1);
    assert_eq!('i', g3.current_char());

    let mut g4 = Buffer::from("two steps forward one step back");
    g4.position(0, 0);
    g4.w(2);
    assert_eq!('f', g4.current_char());
    g4.b(1);
    assert_eq!('s', g4.current_char());
    g4.w(2);
    assert_eq!('o', g4.current_char());
    g4.b(1);
    assert_eq!('f', g4.current_char());
}

#[test]
fn test_advance_word_forward() {
    struct Testcase {
        input: &'static str,
        count: usize,
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
        Testcase {
            input: "the Cat sat",
            count: 1,
            want_char: 's',
        },
        Testcase {
            input: "tHe cat sat",
            count: 1,
            want_char: 'c',
        },
        Testcase {
            input: "a B ] long",
            count: 1,
            want_char: ']',
        },
    ];

    // let _ = env_logger::builder().is_test(true).try_init();
    for (i, tc) in cases.iter().enumerate() {
        let start = tc
            .input
            .chars()
            .position(|c| c.is_ascii_uppercase())
            .unwrap();
        let text = tc.input.to_lowercase();
        let mut b = Buffer::from(text.as_str());
        b.position(0, start);
        b.w(tc.count);
        assert_eq!(tc.want_char, b.current_char(), "Case {i} failed");
    }
}

#[test]
fn test_advance_word_backward() {
    struct Testcase {
        input: &'static str,
        count: usize,
        want_char: char,
    }

    let cases = [
        Testcase {
            input: "abc cde Efg",
            count: 1,
            want_char: 'c',
        },
        Testcase {
            input: "the cat sAt",
            count: 1,
            want_char: 's',
        },
        Testcase {
            input: "the cat Sat",
            count: 2,
            want_char: 't',
        },
        Testcase {
            input: "the cat sAt",
            count: 2,
            want_char: 'c',
        },
        Testcase {
            input: "the cat    sAt",
            count: 2,
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
        let mut b = Buffer::from(text.as_str());
        b.position(0, start);

        println!("Testcase {i}:");
        b.show();
        println!("will call b.prev_word({})", tc.count);

        b.b(tc.count);
        assert_eq!(tc.want_char, b.current_char(), "Case {i} failed");
    }
}

#[test]
fn test_e() {
    struct Testcase {
        input: &'static str,
        count: usize,
        want_char: char,
    }

    let cases = [
        Testcase {
            input: "Abc def ghi",
            count: 1,
            want_char: 'c',
        },
        Testcase {
            input: "aBc def ghi",
            count: 1,
            want_char: 'c',
        },
        Testcase {
            input: "abC def ghi",
            count: 1,
            want_char: 'f',
        },
        Testcase {
            input: "Abc def ghi",
            count: 2,
            want_char: 'f',
        },
        Testcase {
            input: "abc   Def ghi",
            count: 1,
            want_char: 'f',
        },
        Testcase {
            input: "abc Def",
            count: 1,
            want_char: 'f',
        },
    ];

    for (i, tc) in cases.iter().enumerate() {
        let start = tc
            .input
            .chars()
            .position(|c| c.is_ascii_uppercase())
            .unwrap();
        let text = tc.input.to_lowercase();
        let mut b = Buffer::from(text.as_str());
        b.position(0, start);
        b.e(tc.count);
        assert_eq!(tc.want_char, b.current_char(), "Case {i} failed");
    }
}

#[test]
fn test_x() {
    struct Testcase {
        input: &'static str,
        row: usize,
        col: usize,
        count: usize,
        want_text: &'static str,
        want_char: Option<char>,
    }

    let cases = [
        Testcase {
            input: "abcd",
            row: 0,
            col: 0,
            count: 1,
            want_text: "bcd",
            want_char: Some('b'),
        },
        Testcase {
            input: "abcd",
            row: 0,
            col: 2,
            count: 1,
            want_text: "abd",
            want_char: Some('d'),
        },
        Testcase {
            input: "abcde",
            row: 0,
            col: 1,
            count: 3,
            want_text: "ae",
            want_char: None,
        },
        Testcase {
            input: "ab\ncd",
            row: 0,
            col: 1,
            count: 1,
            want_text: "a\ncd",
            want_char: None,
        },
        Testcase {
            input: "a",
            row: 0,
            col: 0,
            count: 1,
            want_text: "",
            want_char: None,
        },
    ];

    for (i, tc) in cases.iter().enumerate() {
        let mut b = Buffer::from(tc.input);
        b.position(tc.row, tc.col);
        b.x(tc.count);
        assert_eq!(tc.want_text, b.text(), "Case {i} text mismatch");
        if let Some(ch) = tc.want_char {
            assert_eq!(ch, b.current_char(), "Case {i} char mismatch");
        }
    }

    // x should not cross newline (double-delete scenario)
    let mut b = Buffer::from("a\ncd");
    b.position(0, 0);
    b.x(1);
    assert_eq!("\ncd", b.text());
    b.x(1);
    assert_eq!("\ncd", b.text());
}

#[test]
fn test_j() {
    let mut b = Buffer::from("abcd\nefg\nhijk\nlmnop");
    b.position(0, 3); // on 'd'
    assert_eq!('d', b.current_char());

    b.j(1);
    assert_eq!('g', b.current_char(), "first j: expected 'g'");

    b.j(1);
    assert_eq!('k', b.current_char(), "second j: expected 'k'");

    b.j(1);
    assert_eq!('o', b.current_char(), "third j: expected 'o'");

    let mut b2 = Buffer::from(
        "-- [ ] open file mikael characters hello world\n- [x] savex mikael is here x\nl\n- hello worlde",
    );
    b2.position(0, 0);
    assert_eq!('-', b2.current_char());
    b2.w(4);
    // b2.position(0, 12);
    assert_eq!('f', b2.current_char());
    b2.j(1);
    assert_eq!('m', b2.current_char());
}

#[test]
fn test_k() {
    // abcd
    // efg
    // hijk
    // lmnop
    //    ^
    let mut b = Buffer::from("abcd\nefg\nhijk\nlmnop");
    b.position(3, 3); // on 'o'
    assert_eq!('o', b.current_char());

    b.k(1);
    assert_eq!('k', b.current_char(), "first k: expected 'k'");

    b.k(1);
    assert_eq!('g', b.current_char(), "second k: expected 'g'");

    b.k(1);
    assert_eq!('d', b.current_char(), "third k: expected 'd'");

    let mut b2 = Buffer::from("long\nx\nshort");
    b2.position(2, 0);
    assert_eq!('s', b2.current_char());
    b2.k(1);
    assert_eq!('x', b2.current_char());
    b2.k(1);
    assert_eq!('l', b2.current_char());

    let mut b3 = Buffer::from("x\nshort");
    // on the r
    b3.position(1, 3);
    assert_eq!('r', b3.current_char());
    b3.k(1);
    assert_eq!('x', b3.current_char());
}

#[test]
fn test_delete_span() {
    let mut b = Buffer::from("the cat sat");
    b.position(0, 5);
    // let span = b.span_for_textobject(Word, Inner, 1);
    // let span = b.span_for_textobject(TextObject(Inner, Word), 1);
    let span = TextObject(Inner, Word).span(&mut b);
    println!("span is {}", span);
    let text = b.delete_span(span, false);
    assert_eq!("the  sat", b.text());
    assert_eq!("cat", text);

    let change = Change {
        span,
        old: text,
        new: "".to_string(),
    };
    b.register_change(change);

    b.undo();
    assert_eq!("the cat sat", b.text());
}

#[test]
fn test_insert_span() {
    let mut b = Buffer::from("abc ghi");
    b.position(0, 5);
    assert_eq!("(0,5)", b.current_position().to_string());
    assert_eq!('h', b.current_char(),);

    b.insert_text(Position { row: 0, col: 4 }, "def  ");
    assert_eq!("abc def  ghi", b.text());
    assert_eq!('h', b.current_char());
}

#[test]
fn test_span() {
    struct Case {
        input: &'static str,
        pos: (usize, usize),
        textobj: TextObject,
        count: usize,
        want: &'static str,
    }
    let cases: Vec<Case> = vec![
        Case {
            input: "(wow\n)",
            pos: (0, 1),
            textobj: TextObject(Around, Paren),
            count: 1,
            want: "(wow\n)",
        },
        Case {
            input: "x{foo}y",
            pos: (0, 1),
            textobj: TextObject(Around, CurlyBracket),
            count: 1,
            want: "{foo}",
        },
        Case {
            input: "x{foo}y",
            pos: (0, 1),
            textobj: TextObject(Inner, CurlyBracket),
            count: 1,
            want: "foo",
        },
        Case {
            input: "x(foo)y",
            pos: (0, 1),
            textobj: TextObject(Inner, Paren),
            count: 1,
            want: "foo",
        },
        Case {
            input: ")x(",
            pos: (0, 1),
            textobj: TextObject(Around, Paren),
            count: 2,
            want: "",
        },
        Case {
            input: "a (xxx) b)",
            pos: (0, 4),
            textobj: TextObject(Around, Paren),
            count: 2,
            want: "",
        },
        Case {
            input: "a (xxx) b",
            pos: (0, 4),
            textobj: TextObject(Around, Paren),
            count: 2,
            want: "",
        },
        Case {
            input: "a (xxx) b",
            pos: (0, 4),
            textobj: TextObject(Around, Paren),
            count: 1,
            want: "(xxx)",
        },
        Case {
            input: "a (inner) b",
            pos: (0, 5),
            textobj: TextObject(Inner, Paren),
            count: 1,
            want: "inner",
        },
        Case {
            input: "the cat     sat",
            pos: (0, 5),
            textobj: TextObject(Inner, Word),
            count: 2,
            want: "cat     ",
        },
        Case {
            input: "the cat sat",
            pos: (0, 5),
            textobj: TextObject(Inner, Word),
            count: 4,
            want: "cat sat",
        },
        Case {
            input: "the cat sat",
            pos: (0, 5),
            textobj: TextObject(Inner, Word),
            count: 3,
            want: "cat sat",
        },
        Case {
            input: "the cat sat",
            pos: (0, 5),
            textobj: TextObject(Inner, Word),
            count: 2,
            want: "cat ",
        },
        Case {
            input: "abba",
            pos: (0, 1),
            textobj: TextObject(Inner, Word),
            count: 3,
            want: "abba",
        },
        Case {
            input: "abba",
            pos: (0, 1),
            textobj: TextObject(Inner, Word),
            count: 2,
            want: "abba",
        },
        Case {
            input: "abba",
            pos: (0, 1),
            textobj: TextObject(Inner, Word),
            count: 1,
            want: "abba",
        },
        Case {
            input: "a.!b!",
            pos: (0, 2),
            textobj: TextObject(Inner, Word),
            count: 1,
            want: ".!",
        },
        Case {
            input: "a.!b!",
            pos: (0, 1),
            textobj: TextObject(Inner, Word),
            count: 2,
            want: ".!b",
        },
        Case {
            input: "a.!b!",
            pos: (0, 1),
            textobj: TextObject(Inner, Word),
            count: 2,
            want: ".!b",
        },
        Case {
            input: "a.!b",
            pos: (0, 1),
            textobj: TextObject(Inner, Word),
            count: 2,
            want: ".!b",
        },
        Case {
            input: "a.!b",
            pos: (0, 1),
            textobj: TextObject(Inner, Word),
            count: 1,
            want: ".!",
        },
        Case {
            input: "a..b",
            pos: (0, 1),
            textobj: TextObject(Inner, Word),
            count: 1,
            want: "..",
        },
        Case {
            input: "a.b",
            pos: (0, 0),
            textobj: TextObject(Inner, Word),
            count: 1,
            want: "a",
        },
        Case {
            input: "a.b",
            pos: (0, 1),
            textobj: TextObject(Inner, Word),
            count: 1,
            want: ".",
        },
        Case {
            input: "the cat sat",
            pos: (0, 5),
            textobj: TextObject(Inner, Word),
            count: 1,
            want: "cat",
        },
    ];

    for (i, tc) in cases.iter().enumerate() {
        let mut b = Buffer::from(tc.input);
        b.position(tc.pos.0, tc.pos.1);
        // let span = b.span_for_textobject(tc.textobj, tc.count);
        let span = tc.textobj.span(&b);
        let got = b.text_for_span(span);
        assert_eq!(
            tc.want, got,
            "\n\nCase {i} failed: input={:?} pos={:?} textobj={:?} count={}\n",
            tc.input, tc.pos, tc.textobj, tc.count
        );
    }
}

#[test]
fn test_text_for_span() {
    let mut b = Buffer::from("the cat sat");
    b.position(0, 5);

    assert_eq!("cat", b.text_for_span(Span::from((0, 4, 0, 7))));

    // let span = b.span_for_textobject(TextObject(Inner, Word), 1);
    let span = TextObject(Inner, Word).span(&b);
    let text = b.text_for_span(span);

    assert_eq!("cat", text);
}

#[test]
fn test_char_at() {
    let sample = "the quick brown\nfox jumps over\nthe lazy dog";
    let chars: Vec<char> = sample.chars().collect();
    let n = chars.len();

    // For N random cursor positions, move the gap there, then verify
    // char_at returns the correct character for every index.
    let positions = [0, 1, 5, 10, 15, 16, 20, 25, 30, 35, n - 1];
    for &cursor_col in &positions {
        // Flatten row/col from the linear index
        let mut row = 0;
        let mut col = 0;
        for &ch in &chars[..cursor_col] {
            if ch == '\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        let mut b = Buffer::from(sample);
        b.position(row, col);

        for i in 0..n {
            assert_eq!(
                Some(chars[i]),
                b.char_at(i),
                "char_at({i}) wrong when cursor at linear pos {cursor_col} (row={row}, col={col})"
            );
        }
        assert_eq!(
            None,
            b.char_at(n),
            "char_at(n) should be None when cursor at {cursor_col}"
        );
    }

    // Also verify char_at works with Position
    let b = Buffer::from(sample);
    assert_eq!(Some('t'), b.char_at(Position { row: 0, col: 0 }));
    assert_eq!(Some('q'), b.char_at(Position { row: 0, col: 4 }));
    assert_eq!(Some('f'), b.char_at(Position { row: 1, col: 0 }));
    assert_eq!(Some('r'), b.char_at(Position { row: 1, col: 13 }));
    assert_eq!(Some('d'), b.char_at(Position { row: 2, col: 9 }));
    assert_eq!(None, b.char_at(Position { row: 5, col: 0 }));

    // Verify to_position is the inverse of CharIndex for Position
    assert_eq!(Some(Position { row: 0, col: 0 }), b.to_position(0));
    assert_eq!(Some(Position { row: 0, col: 4 }), b.to_position(4));
    assert_eq!(Some(Position { row: 1, col: 0 }), b.to_position(16)); // after first '\n'
    assert_eq!(Some(Position { row: 2, col: 11 }), b.to_position(n - 1));
    assert_eq!(None, b.to_position(n + 100));
}
