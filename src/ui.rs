use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::{app::App, highlight::highlight};

impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            // .title("event-driven-async")
            .borders(Borders::TOP | Borders::BOTTOM)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        // read
        let text = self
            .buf
            .text()
            .lines()
            .map(|line| format!("{}\n", line))
            .collect::<Vec<_>>();

        let filetype = self
            .filename
            .extension()
            .map(|ext| ext.to_str())
            .unwrap()
            .unwrap();
        let binding = text.join("");
        let v = highlight(filetype, &binding);

        let mut lines = vec![];
        let mut acc = vec![];
        let mut col = 0;

        for span in &v {
            for c in span.content.chars() {
                // tab
                if c == '\t' {
                    let spaces = 4 - (col % 4);
                    for _ in 0..spaces {
                        acc.push(Span::from(" ").style(span.style));

                        col += 1;
                    }
                    col += spaces;
                    continue;
                }

                if c == '\n' {
                    lines.push(Line::from(acc.clone()));
                    acc.clear();
                    col = 0;
                } else {
                    let str = format!("{}", c);

                    acc.push(Span::from(str).style(span.style));
                    col += 1;
                }
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(block)
            .fg(Color::White)
            .bg(Color::Black);

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        // Title
        Line::from(self.filename.to_str().expect("Invalid filename"))
            .bold()
            .render(header_area, buf);

        // Content
        paragraph.render(inner_area, buf);

        // Cursor
        let cell = &mut buf[(
            inner_area.x + self.buf.col as u16,
            1 + inner_area.y + self.buf.row as u16, // +1 because border at top
        )];
        cell.set_bg(Color::White);

        render_footer(self, footer_area, buf);
    }
}

fn render_footer<'a>(app: &'a App, area: Rect, buf: &'a mut Buffer) {
    let [lhs, mid, rhs] = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Percentage(40),
        Constraint::Percentage(30),
    ])
    .areas(area);

    Line::from(vec![
        Span::from(app.mode.to_string()).fg(Color::Green),
        Span::from(format!(" ({}, {})", app.buf.row, app.buf.col)).fg(Color::DarkGray),
    ])
    .render(lhs, buf);

    Line::from(app.filename.to_str().expect("Invalid filename"))
        .centered()
        .render(mid, buf);

    Line::from(format!("{:<10}", app.cmdbuf.text()).as_str())
        .right_aligned()
        .render(rhs, buf);
}
