use ratatui::prelude::*;
use ratatui::widgets::Block;
use crate::app::App;

pub struct SearchModalWidget<'a> {
    pub app: &'a App,
}

impl Widget for &SearchModalWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .title(" Search ")
            .render(area, buf);
    }
}
