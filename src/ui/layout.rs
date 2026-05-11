use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[allow(dead_code)]
pub fn get_layout(area: Rect) -> [Rect; 5] {
    let chunks = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ],
    ).split(area);

    let rects: Vec<Rect> = chunks.iter().cloned().collect();
    let top = rects[0];
    let body = rects[2];
    let status = rects[1];

    let main_chunks = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(65),
        ],
    ).split(body);

    [top, main_chunks[0], main_chunks[1], status, body]
}
