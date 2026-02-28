use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use crate::app::{App, Mode};

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

        // Command palette
        // format!("{}", &self.mode).italic().render(footer_area, buf);
        render_footer(self).italic().render(footer_area, buf);
    }
}

fn render_footer(app: &App) -> Line<'_> {
    if let Some(msg) = &app.msg {
        return Line::from(msg.clone()).fg(Color::Green);
    }

    match app.mode {
        Mode::ExCommand => Line::from(vec![
            format!(":{}", app.cmdbuf).into(),
            Span::from(" ").bg(Color::White),
        ]),
        _ => format!(
            "{} -- row {} col {} -- `{}`",
            app.mode,
            app.buf.row,
            app.buf.col,
            Span::from(app.buf.current_line()).fg(Color::Red),
        )
        .into(),
    }
}
