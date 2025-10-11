// Calendar interface view

use ratatui::{layout::Rect, Frame};

pub struct CalendarView;

impl CalendarView {
    pub fn new() -> Self {
        CalendarView
    }

    pub fn render(&self, _f: &mut Frame, _area: Rect) {
        // TODO: Implement calendar view
    }
}
