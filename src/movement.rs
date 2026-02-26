// Returns the index of the start of the next word, given a line, a starting index, and a count of
// how many words to skip.
pub fn next_word(line: &str, start: usize, count: usize) -> Option<usize> {
    // line, just starting at 'start'
    let rest: String = line.chars().skip(start).collect();

    let mut delta = line.len() - rest.len();
    let mut i = 0; // number of 
    let mut w = false;
    for c in rest.chars() {
        delta += 1;
        if c.is_whitespace() {
            if !w {
                i += 1;
            }
            w = true;
        } else {
            w = false;
        }
        if i == count {
            return Some(delta);
        }
    }
    None
}

pub fn prev_word(line: &str, start: usize, count: usize) -> Option<usize> {
    let rest: String = line.chars().take(start).collect();

    let mut words_visited = 0;
    let mut chars_back = 0;

    // hello -> [(l, o), (l, l), (e, l), (h, e)]
    for (p, c) in rest.chars().rev().skip(1).zip(rest.chars().rev()) {
        // check for word boundary
        if p.is_whitespace() && !c.is_whitespace() {
            words_visited += 1;
        }
        chars_back += 1;
        if words_visited == count {
            return Some(start - chars_back);
        }
    }

    words_visited += 1; // start of line. that's a word boundary
    chars_back += 1;
    if words_visited == count {
        return Some(start - chars_back);
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_prev_word() {
        struct Testcase {
            input: &'static str,
            count: usize,
            want: &'static str,
        }

        let cases = vec![
            Testcase {
                input: "the cat Sat",
                count: 1,
                want: "cat",
            },
            Testcase {
                input: "the cat sAt",
                count: 1,
                want: "sAt",
            },
            Testcase {
                input: "the cat Sat",
                count: 2,
                want: "the",
            },
            Testcase {
                input: "the cat sAt",
                count: 2,
                want: "cat",
            },
            Testcase {
                input: "the cat    sAt",
                count: 2,
                want: "cat",
            },
            Testcase {
                input: "the cat    sAt",
                count: 8,
                want: "",
            },
        ];

        cases.iter().enumerate().for_each(|(i, tc)| {
            let start = tc
                .input
                .chars()
                .position(|c| c.is_ascii_uppercase())
                .expect("Failed to find start position");

            let want: Option<usize> = if tc.want.is_empty() {
                None
            } else {
                Some(
                    tc.input
                        .find(tc.want)
                        .map(|pos| pos)
                        .expect("Failed to find want position"),
                )
            };

            let got = prev_word(tc.input, start, tc.count);
            assert_eq!(want, got, "Testcase {i} failed");
        });
    }

    #[test]
    fn test_next_word() {
        // The text input. all lowercase except one letter which is going to be
        // the cursor position. We use a substring to find the desired ending word.
        struct Testcase {
            input: &'static str,
            count: usize,
            want: &'static str,
        }

        let cases = vec![
            Testcase {
                input: "The cat sat",
                count: 1,
                want: "cat",
            },
            Testcase {
                input: "The cat sat",
                count: 2,
                want: "sat",
            },
            Testcase {
                input: "The cat sat",
                count: 3,
                want: "",
            },
            Testcase {
                input: "tHe cat sat",
                count: 1,
                want: "cat",
            },
            Testcase {
                input: "a B ] long",
                count: 1,
                want: "]",
            },
        ];

        cases.iter().enumerate().for_each(|(i, tc)| {
            let start = tc
                .input
                .chars()
                .position(|c| c.is_ascii_uppercase())
                .expect("Failed to find start position");

            let want: Option<usize> = if tc.want.is_empty() {
                None
            } else {
                Some(
                    tc.input
                        .find(tc.want)
                        .map(|pos| pos)
                        .expect("Failed to find want position"),
                )
            };

            let got = next_word(tc.input, start, tc.count);
            assert_eq!(want, got, "Testcase {i} failed");
        });
    }
}
