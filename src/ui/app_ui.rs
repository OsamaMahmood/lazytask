use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::config::Config;
use crate::data::models::Task;
use crate::handlers::input::Action;
use crate::taskwarrior::TaskwarriorIntegration;
use crate::ui::components::task_form::{TaskForm, TaskFormResult};
use crate::ui::components::task_list::TaskListWidget;

pub enum AppView {
    TaskList,
    TaskDetail,
    Reports,
    Settings,
    Help,
}

pub struct AppUI {
    config: Config,
    current_view: AppView,
    show_help_bar: bool,
    task_list_widget: TaskListWidget,
    tasks: Vec<Task>,
    task_form: Option<TaskForm>,
}

impl AppUI {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(AppUI {
            config: config.clone(),
            current_view: AppView::TaskList,
            show_help_bar: config.ui.show_help_bar,
            task_list_widget: TaskListWidget::new(),
            tasks: Vec::new(),
            task_form: None,
        })
    }

    pub async fn load_tasks(&mut self, taskwarrior: &TaskwarriorIntegration) -> Result<()> {
        // Load all tasks (not just pending) and sort by entry date (newest first)
        let mut tasks = taskwarrior.list_tasks(None).await?;
        tasks.sort_by(|a, b| b.entry.cmp(&a.entry)); // Newest first
        self.tasks = tasks;
        self.task_list_widget.set_tasks(self.tasks.clone());
        Ok(())
    }

    pub fn has_active_form(&self) -> bool {
        self.task_form.is_some()
    }

    fn task_to_attributes(task: &Task) -> Vec<(String, String)> {
        let mut attributes = Vec::new();

        // Add project if present
        if let Some(ref project) = task.project {
            attributes.push(("project".to_string(), project.clone()));
        }

        // Add priority if present
        if let Some(ref priority) = task.priority {
            let priority_str = match priority {
                crate::data::models::Priority::High => "H",
                crate::data::models::Priority::Medium => "M", 
                crate::data::models::Priority::Low => "L",
            };
            attributes.push(("priority".to_string(), priority_str.to_string()));
        }

        // Add tags if present (taskwarrior format: +tag1 +tag2)
        for tag in &task.tags {
            attributes.push((format!("+{}", tag), "".to_string()));
        }

        // Add due date if present
        if let Some(due) = task.due {
            let due_str = due.format("%Y-%m-%d").to_string();
            attributes.push(("due".to_string(), due_str));
        }

        attributes
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let size = f.size();
        
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),    // Header
                Constraint::Min(0),       // Main content
                Constraint::Length(1),    // Help bar
            ].as_ref())
            .split(size);

        // Draw header
        self.draw_header(f, chunks[0]);

        // Draw main content based on current view
        match &mut self.current_view {
            AppView::TaskList => self.draw_task_list(f, chunks[1]),
            _ => {
                match self.current_view {
                    AppView::TaskDetail => self.draw_task_detail(f, chunks[1]),
                    AppView::Reports => self.draw_reports(f, chunks[1]),
                    AppView::Settings => self.draw_settings(f, chunks[1]),
                    AppView::Help => self.draw_help(f, chunks[1]),
                    _ => {}
                }
            }
        }

        // Draw help bar
        if self.show_help_bar {
            self.draw_help_bar(f, chunks[2]);
        }

        // Draw task form as overlay if open
        if let Some(ref form) = self.task_form {
            form.render(f, size);
        }
    }

    pub async fn handle_action(&mut self, action: Action, taskwarrior: &TaskwarriorIntegration) -> Result<()> {
        // Handle form actions first if form is open
        if let Some(ref mut form) = self.task_form {
            if let Some(result) = form.handle_input(action.clone())? {
                match result {
                    TaskFormResult::Save(task) => {
                        if let Some(task_id) = task.id {
                            // Update existing task
                            let attributes = Self::task_to_attributes(&task);
                            let attributes_refs: Vec<(&str, &str)> = attributes.iter()
                                .map(|(k, v)| (k.as_str(), v.as_str()))
                                .collect();
                            taskwarrior.modify_task(task_id, &attributes_refs).await?;
                        } else {
                            // Add new task
                            let attributes = Self::task_to_attributes(&task);
                            let attributes_refs: Vec<(&str, &str)> = attributes.iter()
                                .map(|(k, v)| (k.as_str(), v.as_str()))
                                .collect();
                            taskwarrior.add_task(&task.description, &attributes_refs).await?;
                        }
                        self.task_form = None;
                        self.load_tasks(taskwarrior).await?;
                    }
                    TaskFormResult::Cancel => {
                        self.task_form = None;
                    }
                }
                return Ok(());
            }
        }

        match action {
            Action::Quit => {
                // This will be handled by the main app loop
            }
            Action::Help => {
                self.current_view = AppView::Help;
            }
            Action::Reports => {
                self.current_view = AppView::Reports;
            }
            Action::Back => {
                if self.task_form.is_some() {
                    self.task_form = None;
                } else {
                    self.current_view = AppView::TaskList;
                }
            }
            Action::MoveUp => {
                if self.task_form.is_none() && matches!(self.current_view, AppView::TaskList) {
                    self.task_list_widget.previous();
                }
            }
            Action::MoveDown => {
                if self.task_form.is_none() && matches!(self.current_view, AppView::TaskList) {
                    self.task_list_widget.next();
                }
            }
            Action::Refresh => {
                self.load_tasks(taskwarrior).await?;
            }
            _ => {
                // Handle other actions based on current view
                if self.task_form.is_none() {
                    match self.current_view {
                        AppView::TaskList => self.handle_task_list_action(action, taskwarrior).await?,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let title = match self.current_view {
            AppView::TaskList => "Taskwarrior TUI - Task List",
            AppView::TaskDetail => "Taskwarrior TUI - Task Detail",
            AppView::Reports => "Taskwarrior TUI - Reports",
            AppView::Settings => "Taskwarrior TUI - Settings",
            AppView::Help => "Taskwarrior TUI - Help",
        };

        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        
        f.render_widget(header, area);
    }

    fn draw_task_list(&mut self, f: &mut Frame, area: Rect) {
        self.task_list_widget.render(f, area);
    }

    fn draw_task_detail(&self, f: &mut Frame, area: Rect) {
        let detail = Paragraph::new("Task Detail View - Coming Soon")
            .block(Block::default().title("Task Detail").borders(Borders::ALL));
        
        f.render_widget(detail, area);
    }

    fn draw_reports(&self, f: &mut Frame, area: Rect) {
        let reports = Paragraph::new("Reports View - Coming Soon")
            .block(Block::default().title("Reports").borders(Borders::ALL));
        
        f.render_widget(reports, area);
    }

    fn draw_settings(&self, f: &mut Frame, area: Rect) {
        let settings = Paragraph::new("Settings View - Coming Soon")
            .block(Block::default().title("Settings").borders(Borders::ALL));
        
        f.render_widget(settings, area);
    }

    fn draw_help(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from("Keyboard Shortcuts:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw("     - Quit application"),
            ]),
            Line::from(vec![
                Span::styled("F1", Style::default().fg(Color::Yellow)),
                Span::raw("    - Show this help"),
            ]),
            Line::from(vec![
                Span::styled("a", Style::default().fg(Color::Yellow)),
                Span::raw("     - Add new task"),
            ]),
            Line::from(vec![
                Span::styled("e", Style::default().fg(Color::Yellow)),
                Span::raw("     - Edit selected task"),
            ]),
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw("     - Mark task as done"),
            ]),
            Line::from(vec![
                Span::styled("Del", Style::default().fg(Color::Yellow)),
                Span::raw("   - Delete selected task"),
            ]),
            Line::from(""),
            Line::from("Press ESC to go back"),
        ];

        let help = Paragraph::new(help_text)
            .block(Block::default().title("Help").borders(Borders::ALL));
        
        f.render_widget(help, area);
    }

    fn draw_help_bar(&self, f: &mut Frame, area: Rect) {
        let help_text = match self.current_view {
            AppView::TaskList => "[a]dd [e]dit [d]one [Del]ete [/]filter [r]eports [q]uit",
            AppView::Help => "[ESC] back",
            _ => "[ESC] back [q]uit",
        };

        let help_bar = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray));
        
        f.render_widget(help_bar, area);
    }

    async fn handle_task_list_action(&mut self, action: Action, taskwarrior: &TaskwarriorIntegration) -> Result<()> {
        match action {
            Action::AddTask => {
                self.task_form = Some(TaskForm::new_task());
            }
            Action::EditTask => {
                if let Some(task) = self.task_list_widget.selected_task() {
                    self.task_form = Some(TaskForm::edit_task(task.clone()));
                }
            }
            Action::DoneTask => {
                if let Some(task) = self.task_list_widget.selected_task() {
                    if let Some(task_id) = task.id {
                        taskwarrior.done_task(task_id).await?;
                        self.load_tasks(taskwarrior).await?;
                    }
                }
            }
            Action::DeleteTask => {
                if let Some(task) = self.task_list_widget.selected_task() {
                    if let Some(task_id) = task.id {
                        taskwarrior.delete_task(task_id).await?;
                        self.load_tasks(taskwarrior).await?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
