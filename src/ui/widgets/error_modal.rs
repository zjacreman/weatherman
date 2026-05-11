use ratatui::prelude::*;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Paragraph};

pub struct ErrorModal<'a> {
    pub message: &'a str,
}

impl Widget for &ErrorModal<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = 60u16;
        let height = 7u16;
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let modal_area = Rect::new(x, y, width, height);

        let block = Block::bordered()
            .title(" Error ")
            .style(Style::default().bg(Color::Rgb(50, 0, 0)).fg(Color::Red))
            .border_set(ratatui::symbols::border::THICK);
        block.render(modal_area, buf);

        let inner = modal_area.inner(Margin::new(1, 2));
        // Truncate message to fit
        let text: String = self.message.chars().take(inner.width.saturating_sub(2) as usize).collect();
        Paragraph::new(text).style(Style::default().fg(Color::White).bg(Color::Rgb(50, 0, 0))).render(inner, buf);

        let hint_y = inner.bottom().saturating_sub(1);
        let hint_text = " Press Esc to dismiss ";
        let line_width = (hint_text.len() as u16).min(inner.width);
        Paragraph::new(hint_text.to_string())
            .style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(50, 0, 0)))
            .render(Rect::new(inner.left(), hint_y, line_width, 1), buf);
    }
}
