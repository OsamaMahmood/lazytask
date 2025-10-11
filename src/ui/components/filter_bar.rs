// Interactive filter builder component

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct FilterBarWidget;

impl FilterBarWidget {
    pub fn new() -> Self {
        FilterBarWidget
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let filter = Paragraph::new("Filter Bar - Coming Soon")
            .block(Block::default().title("Filters").borders(Borders::ALL));
        f.render_widget(filter, area);
    }
}
