use std::{fmt::Display, ops::Index};

use crate::textobject::{Boundary, TextObject};

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
struct Matcher {
    criterias: Vec<Criteria>,
    accept_count: bool,
}

/// Describes a comand pattern, e.g. `w[rite]` matches both `w` and `wr`, ...
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Pattern {
    matchers: Vec<Matcher>,
}

impl Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pattern").unwrap();
        Ok(())
    }
}

/// A single letter in the command. Either encapsulated in a `[...]` or not.
/// If it is, then it's optional. Otherwise, it's required.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
struct Criteria {
    c: char,
    required: bool,
}

impl TryFrom<&str> for Pattern {
    type Error = String;

    fn try_from(mut value: &str) -> Result<Self, Self::Error> {
        let mut criterias = vec![];
        let mut required = true;
        let accept_count = value.starts_with("<count>");
        if accept_count {
            value = value.strip_prefix("<count>").unwrap();
        }

        for c in value.chars() {
            if c == '[' {
                required = false;
            } else if c == ']' {
                required = true;
            } else {
                criterias.push(Criteria { c, required });
            }
        }

        Ok(Self {
            matchers: vec![Matcher {
                criterias,
                accept_count,
            }],
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchResult {
    NoMatch,
    PartialMatch,
    Match,
}

pub trait PatternMatcher {
    fn matches_pattern(&self, text: &str) -> MatchResult;
}

impl Pattern {
    pub fn matches(&self, test: &str) -> MatchResult {
        self.matchers
            .iter()
            .map(|matcher| Self::match_one(matcher, test))
            .max()
            .unwrap_or(MatchResult::NoMatch)
    }

    fn match_one(matcher: &Matcher, mut test: &str) -> MatchResult {
        if matcher.accept_count {
            let (_, stripped) = extract_count(test);
            test = stripped;
        }

        // w[rite] matches w, wr, wri, writ, write
        for (i, c) in test.chars().enumerate() {
            match matcher.criterias.get(i) {
                None => return MatchResult::NoMatch, // test is longer than pattern
                Some(crit) => {
                    if c != crit.c {
                        return MatchResult::NoMatch;
                    }
                    // otherwise they are the same
                }
            }
        }

        // we made it here. That means all match. We check that there are no required
        // input left
        if matcher
            .criterias
            .iter()
            .skip(test.len())
            .all(|crit| !crit.required)
        {
            MatchResult::Match
        } else {
            MatchResult::PartialMatch
        }
    }
}

impl std::ops::BitOr for Pattern {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.matchers.extend(rhs.matchers);
        self
    }
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

// Pattern::Motion
// d<motion>

// or basically: d + motion
// Pattern2::new().Require('d').Optional(Count).

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Motion {
    pub count: Option<usize>,
    pub boundary: Boundary,
    pub object: TextObject,
}

impl Motion {
    pub fn from_cmd(input: &str) -> Option<Self> {
        let (count, rest) = extract_count(input);

        let mut boundary = Boundary::Current;
        let mut object: Option<TextObject> = None;

        let mut it = rest.chars();
        match it.next() {
            Some('a') => boundary = Boundary::Around,
            Some('i') => boundary = Boundary::Inner,
            Some('w') => object = Some(TextObject::Word),
            Some('e') => object = Some(TextObject::End),
            _ => return None,
        }
        match it.next() {
            Some('w') => object = Some(TextObject::Word),
            Some('e') => object = Some(TextObject::End),
            _ => {}
        };

        if let Some(object) = object {
            return Some(Self {
                count,
                boundary,
                object,
            });
        }

        None
    }
}

// impl PatternMatcher for Motion {
//     fn matches_pattern(&self, text: &str) -> MatchResult {
//     }
// }
