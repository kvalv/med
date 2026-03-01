use crate::{
    cmd::pattern::{MatchResult, Motion, Pattern},
    textobject::{Boundary, TextObject},
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
    assert_eq!(Motion::matches("2"), MatchResult::PartialMatch,);
    assert_eq!(
        Some(Motion {
            count: None,
            boundary: Boundary::Inner,
            object: TextObject::Word,
        }),
        Motion::from_cmd("iw").0
    );
    assert_eq!(
        Some(Motion {
            count: None,
            boundary: Boundary::Around,
            object: TextObject::Word,
        }),
        Motion::from_cmd("aw").0
    );
    assert_eq!(
        Some(Motion {
            count: Some(2),
            boundary: Boundary::Current,
            object: TextObject::Word,
        }),
        Motion::from_cmd("2w").0
    );
    assert_eq!(
        Some(Motion {
            count: None,
            boundary: Boundary::Current,
            object: TextObject::Word,
        }),
        Motion::from_cmd("w").0
    );
    assert_eq!(
        Some(Motion {
            count: Some(100),
            boundary: Boundary::Current,
            object: TextObject::Word,
        }),
        Motion::from_cmd("100w").0
    );
    assert_eq!(
        Some(Motion {
            count: Some(23),
            boundary: Boundary::Current,
            object: TextObject::End,
        }),
        Motion::from_cmd("23e").0
    );
}

#[test]
fn test_d_motion() {
    let pat = Pattern::from("d<motion>");
    assert_eq!(MatchResult::Match, pat.matches("diw"));
}
