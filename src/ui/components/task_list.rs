// Task display widget

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::data::models::Task;

pub struct TaskListWidget {
    pub state: ListState,
    tasks: Vec<Task>,
}

impl TaskListWidget {
    pub fn new() -> Self {
        TaskListWidget {
            state: ListState::default(),
            tasks: Vec::new(),
        }
    }

    pub fn set_tasks(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
        if !self.tasks.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tasks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected_task(&self) -> Option<&Task> {
        if let Some(index) = self.state.selected() {
            self.tasks.get(index)
        } else {
            None
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .tasks
            .iter()
            .map(|task| {
                let priority = task.priority.as_ref()
                    .map(|p| p.as_char().to_string())
                    .unwrap_or(" ".to_string());
                
                let project = task.project.as_deref().unwrap_or("");
                
                let due = if let Some(due) = task.due {
                    crate::utils::formatting::format_date(&due)
                } else {
                    "".to_string()
                };

                let text = format!(
                    "{:3} {:3} {} {:10} {}",
                    task.id.unwrap_or(0),
                    project,
                    priority,
                    due,
                    task.description
                );
                
                ListItem::new(text)
            })
            .collect();

        let tasks = List::new(items)
            .block(Block::default().title("Tasks").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(tasks, area, &mut self.state);
    }
}
