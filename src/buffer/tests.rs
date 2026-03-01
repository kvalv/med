use crate::textobject::Boundary::*;
use crate::textobject::TextObject::*;
use std::hint;

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
fn test_d() {
    use Boundary::*;
    use TextObject::*;
    let mut b: Buffer;

    b = Buffer::from("hi\nworld");
    b.position(0, 0);
    b.d(1, Current, Word);
    assert_eq!("\nworld", b.text());
    // return;

    b = Buffer::from("the cat sat");
    b.position(0, 5);
    assert_eq!('a', b.current_char());
    b.d(1, Inner, Word);
    assert_eq!("the  sat", b.text());

    b = Buffer::from("the cat sat");
    b.position(0, 4);
    b.d(1, Inner, Word);
    assert_eq!("the  sat", b.text());

    b = Buffer::from("the cat sat");
    b.position(0, 3); // kills the whitespace
    b.d(1, Inner, Word);
    assert_eq!("thecat sat", b.text());

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
}

#[test]
fn test_undo() {
    let mut b = Buffer::from("the cat sat");
    b.position(0, 5);
    b.d(1, Inner, Word);
    b.undo();
}
