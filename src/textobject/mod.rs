use std::ops::Index;

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Copy)]
pub enum TextObject {
    #[default]
    Paren, // ()
    CurlyBracket, // {}
    Word,
    WordEnd, // e
    // Paren, // ()
    // Brack, // <>
    Back, // b
}

impl TextObject {
    pub fn open_symbol(&self) -> Option<char> {
        match self {
            TextObject::Paren => Some('('),
            TextObject::CurlyBracket => Some('{'),
            _ => None,
        }
    }
    pub fn close_symbol(&self) -> Option<char> {
        match self {
            TextObject::Paren => Some(')'),
            TextObject::CurlyBracket => Some('}'),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Copy)]
pub enum Boundary {
    #[default]
    Current, // From current location. I would like to use 'None', but that would be bad.
    Inner,
    // includes whitespace . Prefers whitespace ahead. If not possible (eg due to different
    // word type) then uses the other side.
    // Not really clear to me the logic of how this is in vim...
    Around,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
struct Matcher {
    criterias: Vec<Criteria>,
    accept_count: bool,
}

/// Describes a comand pattern, e.g. `w[rite]` matches both `w` and `wr`, ...
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Pattern {
    verb: Option<char>,
    matchers: Vec<Matcher>,
}

/// A single letter in the command. Either encapsulated in a `[...]` or not.
/// If it is, then it's optional. Otherwise, it's required.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
struct Criteria {
    c: char,
    required: bool,
}

impl From<&str> for Pattern {
    fn from(mut value: &str) -> Self {
        if value.contains("<motion>") {
            let verb = value.chars().next().unwrap();
            if value.chars().skip(1).collect::<String>() != "<motion>" {
                panic!("does not end with <motion>");
            }
            return Self {
                verb: Some(verb),
                matchers: vec![],
            };
        }

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

        Self {
            verb: None,
            matchers: vec![Matcher {
                criterias,
                accept_count,
            }],
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchResult {
    NoMatch,
    PartialMatch,
    Match,
}

impl Pattern {
    pub fn matches(&self, test: &str) -> MatchResult {
        if let Some(c) = self.verb {
            // then we'll instead match by <count>v<motion>
            if !test.starts_with(c) {
                return MatchResult::NoMatch;
            }
            let rest: String = test.chars().skip(1).collect();
            if rest.is_empty() {
                return MatchResult::PartialMatch;
            }
            let got = match_textobject(&rest);
            return got;
        }

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

// textobject: a|i <object>

pub fn match_textobject(input: &str) -> MatchResult {
    let (m, _) = parse_textobject(input);
    if m.is_some() {
        return MatchResult::Match;
    }

    let (count, rest) = extract_count(input);

    if count.is_some() && count.unwrap() > 0 {
        return MatchResult::PartialMatch;
    }

    match rest.chars().next() {
        Some('a' | 'i') => MatchResult::PartialMatch,
        _ => MatchResult::NoMatch,
    }
}

pub fn parse_textobject(input: &str) -> (Option<(TextObject, Boundary, Option<usize>)>, usize) {
    let (count, rest) = extract_count(input);
    let mut consumed = count.iter().len();

    let mut boundary = Boundary::Current;
    let mut object: Option<TextObject> = None;

    let mut it = rest.chars();
    match it.next() {
        Some('a') => boundary = Boundary::Around,
        Some('i') => boundary = Boundary::Inner,
        Some('w') => object = Some(TextObject::Word),
        Some('e') => object = Some(TextObject::WordEnd),
        Some('(') => object = Some(TextObject::Paren),
        Some('{') => object = Some(TextObject::CurlyBracket),
        _ => return (None, 0),
    }
    consumed += 1;

    match it.next() {
        Some('w') => object = Some(TextObject::Word),
        Some('e') => object = Some(TextObject::WordEnd),
        Some('(') => object = Some(TextObject::Paren),
        Some('{') => object = Some(TextObject::CurlyBracket),
        _ => {}
    };

    if let Some(object) = object {
        consumed += 1;
        return (Some((object, boundary, count)), consumed);
    }

    (None, 0)
}
