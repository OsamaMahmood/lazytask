// Task edit form component

use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct TaskDetailWidget;

impl TaskDetailWidget {
    pub fn new() -> Self {
        TaskDetailWidget
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let detail = Paragraph::new("Task Detail Widget - Coming Soon")
            .block(Block::default().title("Task Detail").borders(Borders::ALL));
        f.render_widget(detail, area);
    }
}
