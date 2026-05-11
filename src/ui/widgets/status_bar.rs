use ratatui::prelude::*;
use ratatui::widgets::Block;
use crate::app::App;

pub struct StatusWidget<'a> {
    pub app: &'a App,
}

impl Widget for &StatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .title(" Controls: S=search | 1/2/3=tabs | U=unit | R=refresh | Esc=clear | Ctrl+C=quit ")
            .render(area, buf);
    }
}
