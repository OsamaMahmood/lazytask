// Configuration UI view

use ratatui::{layout::Rect, Frame};

pub struct SettingsView;

impl SettingsView {
    pub fn new() -> Self {
        SettingsView
    }

    pub fn render(&self, _f: &mut Frame, _area: Rect) {
        // TODO: Implement settings view
    }
}
