use crate::textobject::{
    Boundary, MatchResult, Pattern, TextObject, match_textobject, parse_textobject,
};

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
        Some((TextObject::End, Boundary::Current, Some(23))),
        parse_textobject("23e").0
    );
}

#[test]
fn test_d_motion() {
    let pat = Pattern::from("d<motion>");
    assert_eq!(MatchResult::Match, pat.matches("diw"));
}
