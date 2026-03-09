use super::*;
use crate::textobject::{
    Boundary, MatchResult, Pattern, TextObject, match_textobject, parse_textobject,
};

#[test]
fn test_parse_from_cmd() {
    let mut buf = CmdBuf::new();
    buf.push('d');
    buf.push('i');
    assert_eq!(buf.parse(), None);
    buf.push('w'); // diw
    assert_eq!(
        buf.parse(),
        Some(Command::Delete(Delete {
            count: None,
            text_object: Some((Boundary::Inner, TextObject::Word)),
            movement: None,
            linewise: false,
        }))
    );
    assert_eq!(buf.parse(), None);
}

#[test]
fn test_parse() {
    let t = |input, want| {
        let got = parser().parse(input).unwrap();
        assert_eq!(got, want);
    };

    t(
        "4e",
        Command::Movement(Movement {
            count: Some(4),
            char: 'e',
        }),
    );
    t(
        "w",
        Command::Movement(Movement {
            count: None,
            char: 'w',
        }),
    );
    t(
        "diw",
        Command::Delete(Delete {
            count: None,
            text_object: Some((Boundary::Inner, TextObject::Word)),
            movement: None,
            linewise: false,
        }),
    );
    t(
        "j",
        Command::Movement(Movement {
            count: None,
            char: 'j',
        }),
    );
    t(
        "4d3j",
        Command::Delete(Delete {
            count: Some(4),
            movement: Some(Movement {
                count: Some(3),
                char: 'j',
            }),
            ..Default::default()
        }),
    );
    t(
        "dj",
        Command::Delete(Delete {
            count: None,
            movement: Some(Movement {
                count: None,
                char: 'j',
            }),
            ..Default::default()
        }),
    );
    t(
        "0k",
        Command::Movement(Movement {
            count: Some(0),
            char: 'k',
        }),
    );
    t(
        "2j",
        Command::Movement(Movement {
            count: Some(2),
            char: 'j',
        }),
    );
    t(
        "j",
        Command::Movement(Movement {
            count: None,
            char: 'j',
        }),
    );
}

#[test]
fn test_movement_span() {
    let t = |input: &str, cursors: Position, count: usize, char: char, want: &str| {
        let mut buf = Buffer::from(input);
        buf.position(cursors.row, cursors.col);
        let m = Movement {
            count: Some(count),
            char,
        };
        let got = m.span(&mut buf);
        let got_text = buf.text_for_span(got);

        assert_eq!(got_text, want, "wanted '{}', got '{}'", want, got_text);
    };

    t("i", Position { row: 0, col: 0 }, 1, 'b', "");
    t("i ]a] move", Position { row: 0, col: 6 }, 4, 'b', "i ]a] ");
    t("i ]a] move", Position { row: 0, col: 6 }, 3, 'b', "]a] ");
    t("i ]a] move", Position { row: 0, col: 6 }, 2, 'b', "a] ");
    t("i ]a] move", Position { row: 0, col: 6 }, 1, 'b', "] ");
    t("i ]] move", Position { row: 0, col: 5 }, 1, 'b', "]] ");
    t("the cat sat", Position { row: 0, col: 4 }, 1, 'b', "the ");
    t("the cat sat", Position { row: 0, col: 5 }, 1, 'b', "c");
    t("the cat sat", Position { row: 0, col: 5 }, 1, 'w', "at ");
}

#[test]
fn test_pattern_write() {
    let pat = Pattern::from("w[rite]");
    assert_eq!(MatchResult::Match, pat.matches("w"));
    assert_eq!(MatchResult::Match, pat.matches("wr"));
    assert_eq!(MatchResult::Match, pat.matches("wri"));
    assert_eq!(MatchResult::Match, pat.matches("writ"));
    assert_eq!(MatchResult::Match, pat.matches("write"));
    assert_eq!(MatchResult::NoMatch, pat.matches("writex"));
}

#[test]
fn test_with_count() {
    let pat = Pattern::from("<count>g[g]");
    assert_eq!(MatchResult::Match, pat.matches("g"));
    assert_eq!(MatchResult::Match, pat.matches("32g"));
    assert_eq!(MatchResult::Match, pat.matches("32gg"));
    assert_eq!(MatchResult::NoMatch, pat.matches("32ggg"));
    assert_eq!(MatchResult::PartialMatch, pat.matches("32"));
}

#[test]
fn test_motion() {
    assert_eq!(match_textobject("2"), MatchResult::PartialMatch);
    assert_eq!(
        Some((TextObject::Word, Boundary::Inner, None)),
        parse_textobject("iw").0
    );
    assert_eq!(
        Some((TextObject::Word, Boundary::Around, None)),
        parse_textobject("aw").0
    );
    assert_eq!(
        Some((TextObject::Word, Boundary::Current, Some(2))),
        parse_textobject("2w").0
    );
    assert_eq!(
        Some((TextObject::Word, Boundary::Current, None)),
        parse_textobject("w").0
    );
    assert_eq!(
        Some((TextObject::Word, Boundary::Current, Some(100))),
        parse_textobject("100w").0
    );
    assert_eq!(
        Some((TextObject::WordEnd, Boundary::Current, Some(23))),
        parse_textobject("23e").0
    );
}

#[test]
fn test_d_motion() {
    let pat = Pattern::from("d<motion>");
    assert_eq!(MatchResult::Match, pat.matches("diw"));
}
