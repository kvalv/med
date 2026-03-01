use crate::cmd::pattern::{MatchResult, Pattern};

#[test]
fn test_pattern_write() {
    let pat = Pattern::try_from("w[rite]").expect("Failed to parse pattern");
    assert_eq!(MatchResult::Match, pat.matches("w"));
    assert_eq!(MatchResult::Match, pat.matches("wr"));
    assert_eq!(MatchResult::Match, pat.matches("wri"));
    assert_eq!(MatchResult::Match, pat.matches("writ"));
    assert_eq!(MatchResult::Match, pat.matches("write"));
    assert_eq!(MatchResult::NoMatch, pat.matches("writex"));
}

#[test]
fn test_with_count() {
    let pat = Pattern::try_from("<count>g[g]").unwrap();
    assert_eq!(MatchResult::Match, pat.matches("g"));
    assert_eq!(MatchResult::Match, pat.matches("32g"));
    assert_eq!(MatchResult::Match, pat.matches("32gg"));
    assert_eq!(MatchResult::NoMatch, pat.matches("32ggg"));
    assert_eq!(MatchResult::PartialMatch, pat.matches("32"));
}
