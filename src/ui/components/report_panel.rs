// Statistics and charts component

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct ReportPanelWidget;

impl ReportPanelWidget {
    pub fn new() -> Self {
        ReportPanelWidget
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let reports = Paragraph::new("Report Panel - Coming Soon")
            .block(Block::default().title("Reports").borders(Borders::ALL));
        f.render_widget(reports, area);
    }
}
