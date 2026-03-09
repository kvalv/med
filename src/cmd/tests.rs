use super::*;
use crate::textobject::{Object, Variant};

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
            text_object: Some((Variant::Inner, Object::Word)),
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
            text_object: Some((Variant::Inner, Object::Word)),
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
