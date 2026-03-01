use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::app::App;

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

        let v: Vec<_> = text
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if self.buf.row == i {
                    // split into three?
                    // let mut spans: Vec<Span> ;
                    let mut spans = vec![];
                    let lhs: String = line.chars().take(self.buf.col).collect();
                    if !lhs.is_empty() {
                        spans.push(Span::from(lhs));
                    }
                    if let Some(c) = line.chars().nth(self.buf.col) {
                        if c == '\n' {
                            spans.push(Span::from(" ").bg(Color::Red));
                        } else {
                            spans.push(Span::from(String::from(c)).bg(Color::Red))
                        }
                    }
                    let rhs: String = line.chars().skip(self.buf.col + 1).collect();
                    if !rhs.is_empty() {
                        spans.push(Span::from(rhs));
                    }
                    Line::from(spans)
                } else {
                    Line::from(line.clone())
                }
            })
            .collect();

        let paragraph = Paragraph::new(v)
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
        Line::from(self.filename.clone())
            .bold()
            .render(header_area, buf);

        // Content
        paragraph.render(inner_area, buf);

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

    Line::from(app.filename.as_str())
        .centered()
        .render(mid, buf);

    Line::from(format!("{:<10}", app.cmdbuf.text()).as_str())
        .right_aligned()
        .render(rhs, buf);
}
