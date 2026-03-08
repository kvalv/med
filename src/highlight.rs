use ratatui::text::Span;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    None,
    Keyword,
    Func,
    String,
    Number,
    Comment,
    Type,
    Operator,
    Property,
    Constant,
    Escape,
}

fn capture_class(name: &str) -> Capture {
    match name {
        "keyword" | "constant.builtin" => Capture::Keyword,
        "function" | "function.method" | "function.builtin" | "function.call" => Capture::Func,
        "string" | "string.special" | "text.literal" => Capture::String,
        "number" | "float" => Capture::Number,
        "comment" => Capture::Comment,
        "type" | "type.builtin" => Capture::Type,
        "operator" | "punctuation.delimiter" => Capture::Operator,
        "property" | "text.reference" => Capture::Property,
        "constant" => Capture::Constant,
        "escape" | "string.escape" => Capture::Escape,
        "text.emphasis" => Capture::Comment,        // italic
        "text.strong" => Capture::Keyword,          // bold-ish (red)
        "text.title" => Capture::Keyword,           // headings
        "punctuation.special" => Capture::Operator, // heading markers, list bullets
        "text.uri" => Capture::String,
        _ => Capture::None,
    }
}

fn get_config(lang: &str) -> Option<Vec<(Language, String)>> {
    match lang {
        "go" => Some(vec![(
            tree_sitter_go::LANGUAGE.into(),
            tree_sitter_go::HIGHLIGHTS_QUERY.to_string(),
        )]),
        "md" => Some(vec![
            (
                tree_sitter_md::LANGUAGE.into(),
                tree_sitter_md::HIGHLIGHT_QUERY_BLOCK.to_string(),
            ),
            (
                tree_sitter_md::INLINE_LANGUAGE.into(),
                tree_sitter_md::HIGHLIGHT_QUERY_INLINE.to_string(),
            ),
        ]),
        _ => None,
    }
}

pub fn highlight<'a>(lang: &'a str, code: &'a str) -> Vec<Span<'a>> {
    let Some(configs) = get_config(lang) else {
        return vec![Span::from(code)];
    };

    let mut ranges: Vec<(usize, usize, Capture)> = Vec::new();

    for (language, query_src) in configs {
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .expect("language version compatible");
        let Some(tree) = parser.parse(code.as_bytes(), None) else {
            continue;
        };
        let Ok(query) = Query::new(&language, &query_src) else {
            continue;
        };

        let capture_map: Vec<Capture> = query
            .capture_names()
            .iter()
            .map(|name| capture_class(name))
            .collect();

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), code.as_bytes());
        while let Some(m) = matches.next() {
            for capture in m.captures {
                let class = capture_map[capture.index as usize];
                if class != Capture::None {
                    let start = capture.node.start_byte();
                    let end = capture.node.end_byte().min(code.len());
                    ranges.push((start, end, class));
                }
            }
        }
    }

    ranges.sort_by_key(|&(s, _, _)| s);

    // Build spans from consecutive capture groups, filling gaps with unstyled text
    let mut result = Vec::new();
    let mut pos = 0;
    for (start, end, class) in ranges {
        if start < pos {
            continue; // overlapping capture, skip
        }
        if pos < start {
            result.push(Span::from(code[pos..start].to_string()));
        }
        result.push(Span::from(code[start..end].to_string()).style(colorscheme(&class)));
        pos = end;
    }
    if pos < code.len() {
        result.push(Span::from(code[pos..].to_string()));
    }

    result
}

// gruvbox
fn colorscheme(cap: &Capture) -> ratatui::style::Style {
    use ratatui::style::*;
    match cap {
        Capture::None => todo!(),
        Capture::Keyword => Style::default().fg(Color::from_u32(0xfb4934)),
        Capture::Func => Style::default().fg(Color::from_u32(0xb8bb26)),
        Capture::String => Style::default().fg(Color::from_u32(0xb8bb26)),
        Capture::Number => Style::default().fg(Color::from_u32(0xd3869b)),
        Capture::Comment => Style::default()
            .fg(Color::from_u32(0x928374))
            .add_modifier(Modifier::ITALIC),
        Capture::Type => Style::default().fg(Color::from_u32(0xfabd2f)),
        Capture::Operator => Style::default().fg(Color::from_u32(0xfe8019)),
        Capture::Property => Style::default().fg(Color::from_u32(0x83a598)),
        Capture::Constant => Style::default().fg(Color::from_u32(0xd3869b)),
        Capture::Escape => Style::default()
            .fg(Color::from_u32(0xfe8019))
            .add_modifier(Modifier::BOLD),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::*;

    #[test]
    fn go_highlight() {
        let spans = highlight("go", "var x = 42 // count\n");

        let color_for_number = Some(Color::from_u32(0xd3869b));

        assert!(
            spans
                .iter()
                .any(|span| span.content == "42" && span.style.fg == color_for_number)
        );
    }
}
