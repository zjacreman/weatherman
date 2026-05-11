use ratatui::prelude::*;
use ratatui::widgets::Block;
use crate::app::App;

pub struct DailyWidget<'a> {
    pub app: &'a App,
}

impl Widget for &DailyWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::bordered()
            .title(" Daily Forecast ")
            .render(area, buf);
    }
}
