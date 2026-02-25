use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget},
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
            .title("event-driven-async")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        // read
        let text = self.buf.clone();

        let v: Vec<_> = text
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if self.cursor.row == i {
                    // split into three?
                    // let mut spans: Vec<Span> ;
                    let mut spans = vec![];
                    let lhs: String = line.chars().take(self.cursor.col).collect();
                    if !lhs.is_empty() {
                        spans.push(Span::from(lhs));
                    }
                    if let Some(c) = line.chars().nth(self.cursor.col) {
                        spans.push(Span::from(String::from(c)).bg(Color::Red))
                    }
                    let rhs: String = line.chars().skip(self.cursor.col + 1).collect();
                    if !rhs.is_empty() {
                        spans.push(Span::from(rhs));
                    }
                    Line::from(spans)
                } else {
                    Line::from(line.clone()).fg(Color::Blue)
                }
            })
            .collect();

        let paragraph = Paragraph::new(v)
            .block(block)
            .fg(Color::White)
            .bg(Color::Black);

        paragraph.render(area, buf);
    }
}
