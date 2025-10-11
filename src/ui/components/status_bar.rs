// Status and shortcuts component

use ratatui::{
    layout::Rect,
    widgets::Paragraph,
    Frame,
};

pub struct StatusBarWidget;

impl StatusBarWidget {
    pub fn new() -> Self {
        StatusBarWidget
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let status = Paragraph::new("[a]dd [e]dit [d]one [Del]ete [/]filter [r]eports [q]uit");
        f.render_widget(status, area);
    }
}
