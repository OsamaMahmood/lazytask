// Primary task list view

use ratatui::{
    layout::Rect,
    Frame,
};

use crate::ui::components::task_list::TaskListWidget;

pub struct MainView {
    task_list: TaskListWidget,
}

impl MainView {
    pub fn new() -> Self {
        MainView {
            task_list: TaskListWidget::new(),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        self.task_list.render(f, area);
    }
}
