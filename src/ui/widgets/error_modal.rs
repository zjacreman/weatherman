use ratatui::prelude::*;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Paragraph};

pub struct ErrorModal<'a> {
    pub message: &'a str,
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

impl Widget for &ErrorModal<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = 60u16;
        let inner_text_width = width.saturating_sub(6) as usize; // borders + margin

        let wrapped_lines = word_wrap(self.message, inner_text_width);
        let text_height = wrapped_lines.len() as u16;
        let height = (text_height + 4).min(area.height.saturating_sub(2)); // +4: borders + hint + padding

        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let modal_area = Rect::new(x, y, width, height);

        let block = Block::bordered()
            .title(" Error ")
            .style(Style::default().bg(Color::Rgb(50, 0, 0)).fg(Color::Red))
            .border_set(ratatui::symbols::border::THICK);
        block.render(modal_area, buf);

        let inner = modal_area.inner(Margin::new(1, 1));

        let bg_style = Style::default().fg(Color::White).bg(Color::Rgb(50, 0, 0));
        let lines: Vec<Line> = wrapped_lines
            .iter()
            .map(|l| Line::from(Span::styled(l.clone(), bg_style)))
            .collect();
        let text_height = lines.len() as u16;
        let text_area = Rect::new(inner.left(), inner.top(), inner.width, text_height.min(inner.height.saturating_sub(1)));
        Paragraph::new(lines).render(text_area, buf);

        let hint_y = inner.bottom().saturating_sub(1);
        let hint_text = " Press Esc to dismiss ";
        let line_width = (hint_text.len() as u16).min(inner.width);
        Paragraph::new(hint_text.to_string())
            .style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(50, 0, 0)))
            .render(Rect::new(inner.left(), hint_y, line_width, 1), buf);
    }
}
