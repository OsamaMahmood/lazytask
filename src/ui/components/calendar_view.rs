// Calendar widget component

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct CalendarWidget;

impl CalendarWidget {
    pub fn new() -> Self {
        CalendarWidget
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let calendar = Paragraph::new("Calendar Widget - Coming Soon")
            .block(Block::default().title("Calendar").borders(Borders::ALL));
        f.render_widget(calendar, area);
    }
}
