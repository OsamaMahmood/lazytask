// Modal dialog component

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct ModalDialogWidget;

impl ModalDialogWidget {
    pub fn new() -> Self {
        ModalDialogWidget
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let dialog = Paragraph::new("Modal Dialog - Coming Soon")
            .block(Block::default().title("Dialog").borders(Borders::ALL));
        f.render_widget(Clear, area);
        f.render_widget(dialog, area);
    }
}
