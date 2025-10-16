use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::config::Config;
use crate::data::models::Task;
use crate::handlers::input::Action;
use crate::taskwarrior::TaskwarriorIntegration;
use crate::ui::components::task_form::{TaskForm, TaskFormResult};
use crate::ui::views::main_view::MainView;
use crate::ui::views::reports_view::ReportsView;

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
    main_view: MainView,
    reports_view: ReportsView,
    tasks: Vec<Task>,
    filtered_tasks: Vec<Task>,
    task_form: Option<TaskForm>,
    // Track the task UUID to preserve selection after operations
    preserve_selection_uuid: Option<String>,
}

impl AppUI {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(AppUI {
            config: config.clone(),
            current_view: AppView::TaskList,
            show_help_bar: config.ui.show_help_bar,
            main_view: MainView::new(),
            reports_view: ReportsView::new(),
            tasks: Vec::new(),
            filtered_tasks: Vec::new(),
            task_form: None,
            preserve_selection_uuid: None,
        })
    }

    pub async fn load_tasks(&mut self, taskwarrior: &TaskwarriorIntegration) -> Result<()> {
        // Load all tasks (not just pending) and sort by entry date (newest first)
        let mut tasks = taskwarrior.list_tasks(None).await?;
        tasks.sort_by(|a, b| b.entry.cmp(&a.entry)); // Newest first
        self.tasks = tasks.clone();
        
        // Update available filters in main view
        self.main_view.update_available_filters(&self.tasks);
        
        // Update reports view with all tasks
        self.reports_view.update_tasks(tasks);
        
        self.apply_filters();
        Ok(())
    }

    fn apply_filters(&mut self) {
        // Apply custom filters based on selections
        self.filtered_tasks = self.tasks
            .iter()
            .filter(|task| self.main_view.matches_filters(task))
            .cloned()
            .collect();
        
        // Use preserved selection if available
        let preserve_uuid = self.preserve_selection_uuid.as_deref();
        self.main_view.set_tasks_with_preserved_selection(self.filtered_tasks.clone(), preserve_uuid);
        
        // Clear the preserve UUID after using it
        self.preserve_selection_uuid = None;
    }

    pub fn has_active_form(&self) -> bool {
        self.task_form.is_some() || self.main_view.is_filter_focused()
    }

    fn task_to_attributes(task: &Task) -> Vec<(String, String)> {
        let mut attributes = Vec::new();

        // Add description (this was missing!)
        attributes.push(("description".to_string(), task.description.clone()));

        // Add project if present, otherwise clear it
        if let Some(ref project) = task.project {
            attributes.push(("project".to_string(), project.clone()));
        } else {
            attributes.push(("project".to_string(), "".to_string()));
        }

        // Add priority if present, otherwise clear it
        if let Some(ref priority) = task.priority {
            let priority_str = match priority {
                crate::data::models::Priority::High => "H",
                crate::data::models::Priority::Medium => "M", 
                crate::data::models::Priority::Low => "L",
            };
            attributes.push(("priority".to_string(), priority_str.to_string()));
        } else {
            attributes.push(("priority".to_string(), "".to_string()));
        }

        // Handle tags: First clear all tags, then add new ones
        // Clear all existing tags
        attributes.push(("tags".to_string(), "".to_string()));
        
        // Add new tags (taskwarrior format: +tag1 +tag2)
        for tag in &task.tags {
            attributes.push((format!("+{}", tag), "".to_string()));
        }

        // Add due date if present, otherwise clear it
        if let Some(due) = task.due {
            let due_str = due.format("%Y-%m-%d").to_string();
            attributes.push(("due".to_string(), due_str));
        } else {
            attributes.push(("due".to_string(), "".to_string()));
        }

        attributes
    }


    pub fn draw(&mut self, f: &mut Frame) {
        let size = f.size();
        
        // Create responsive dashboard layout that adapts to window size
        let terminal_height = size.height;
        
        // Responsive header/footer sizing based on terminal height
        let (header_size, footer_size) = if terminal_height < 20 {
            (2, 2) // Very small terminals
        } else if terminal_height < 30 {
            (3, 2) // Small terminals  
        } else {
            (3, 3) // Normal/large terminals
        };

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_size),    // Responsive header
                Constraint::Min(10),                // Content area (always minimum 10 lines)
                Constraint::Length(footer_size),    // Responsive footer
            ])
            .split(size);

        // Draw header
        self.draw_header(f, main_chunks[0]);

        // Draw main content based on current view
        match self.current_view {
            AppView::TaskList => {
                // Delegate to main view for task list rendering
                self.main_view.render(f, main_chunks[1], size.width);
            }
            AppView::TaskDetail => self.draw_task_detail(f, main_chunks[1]),
            AppView::Reports => self.draw_reports(f, main_chunks[1]),
            AppView::Settings => self.draw_settings(f, main_chunks[1]),
            AppView::Help => self.draw_help(f, main_chunks[1]),
        }

        // Draw footer with panel boundaries
        self.draw_footer_panel(f, main_chunks[2]);

        // Draw task form as overlay if open
        if let Some(ref form) = self.task_form {
            form.render(f, size);
        }
    }

    pub async fn handle_action(&mut self, action: Action, taskwarrior: &TaskwarriorIntegration) -> Result<()> {
        // Remove old filter handling that was intercepting actions

        // Handle form actions if form is open
        if let Some(ref mut form) = self.task_form {
            if let Some(result) = form.handle_input(action.clone())? {
                match result {
                    TaskFormResult::Save(task) => {
                        if let Some(task_id) = task.id {
                            // Update existing task - preserve selection on the same task
                            self.preserve_selection_uuid = Some(task.uuid.clone());
                            
                            let attributes = Self::task_to_attributes(&task);
                            let attributes_refs: Vec<(&str, &str)> = attributes.iter()
                                .map(|(k, v)| (k.as_str(), v.as_str()))
                                .collect();
                            
                            taskwarrior.modify_task(task_id, &attributes_refs).await?;
                        } else {
                            // Add new task - we'll need to find the newly created task by description
                            // For now, preserve current selection or go to newest (first in list)
                            self.preserve_selection_uuid = self.main_view.selected_task_uuid();
                            
                            let attributes = Self::task_to_attributes(&task);
                            let attributes_refs: Vec<(&str, &str)> = attributes.iter()
                                .map(|(k, v)| (k.as_str(), v.as_str()))
                                .collect();
                            let _new_task_id = taskwarrior.add_task(&task.description, &attributes_refs).await?;
                            
                            // For new tasks, we'll select the first task (newest) since tasks are sorted by entry date
                            self.preserve_selection_uuid = None; // Let it go to newest task
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
            Action::Context => {
                // Toggle calendar mode when in Reports view
                if matches!(self.current_view, AppView::Reports) {
                    self.reports_view.toggle_mode();
                }
            }
            Action::Back => {
                if self.task_form.is_some() {
                    self.task_form = None;
                } else if matches!(self.current_view, AppView::TaskList) && self.main_view.is_filter_focused() {
                    // Single ESC to exit filter mode (only in TaskList view)
                    self.main_view.exit_filter_mode();
                    self.apply_filters(); // Apply filters when exiting
                } else {
                    self.current_view = AppView::TaskList;
                }
            }
            Action::MoveUp => {
                if matches!(self.current_view, AppView::TaskList) && self.main_view.is_filter_focused() {
                    self.main_view.handle_filter_navigation_up();
                } else if matches!(self.current_view, AppView::Reports) && self.reports_view.is_calendar_mode() {
                    // Navigate date backwards by one week in calendar mode
                    self.reports_view.navigate_date(crate::ui::views::reports_view::DateNavigation::PrevWeek);
                } else if self.task_form.is_none() && matches!(self.current_view, AppView::TaskList) {
                    self.main_view.previous_task();
                }
            }
            Action::MoveDown => {
                if matches!(self.current_view, AppView::TaskList) && self.main_view.is_filter_focused() {
                    self.main_view.handle_filter_navigation_down();
                } else if matches!(self.current_view, AppView::Reports) && self.reports_view.is_calendar_mode() {
                    // Navigate date forward by one week in calendar mode
                    self.reports_view.navigate_date(crate::ui::views::reports_view::DateNavigation::NextWeek);
                } else if self.task_form.is_none() && matches!(self.current_view, AppView::TaskList) {
                    self.main_view.next_task();
                }
            }
            Action::MoveLeft => {
                if matches!(self.current_view, AppView::Reports) && self.reports_view.is_calendar_mode() {
                    // Navigate date backwards by one day in calendar mode
                    self.reports_view.navigate_date(crate::ui::views::reports_view::DateNavigation::PrevDay);
                }
            }
            Action::MoveRight => {
                if matches!(self.current_view, AppView::Reports) && self.reports_view.is_calendar_mode() {
                    // Navigate date forward by one day in calendar mode
                    self.reports_view.navigate_date(crate::ui::views::reports_view::DateNavigation::NextDay);
                }
            }
            Action::Refresh => {
                self.load_tasks(taskwarrior).await?;
            }
            Action::Filter => {
                // Only allow filter toggle in TaskList view
                if matches!(self.current_view, AppView::TaskList) {
                    self.main_view.toggle_filter_focus();
                    if !self.main_view.is_filter_focused() {
                        // Exiting filter mode - apply filters
                        self.apply_filters();
                    }
                }
            }
            Action::Tab => {
                // Only handle Tab for filter navigation in TaskList view
                if matches!(self.current_view, AppView::TaskList) && self.main_view.is_filter_focused() {
                    self.main_view.next_filter_section();
                }
            }
            _ => {
                // Handle filter actions if filters are focused AND in TaskList view
                if matches!(self.current_view, AppView::TaskList) && self.main_view.is_filter_focused() {
                    // Don't pass navigation actions to handle_filter_action - they're handled above
                    match action {
                        Action::MoveUp | Action::MoveDown => {
                            // Already handled above, do nothing
                        }
                        Action::Space => {
                            self.main_view.toggle_current_selection();
                            self.apply_filters();
                        }
                        Action::Character(c) => {
                            self.main_view.handle_search_character(c);
                            self.apply_filters();
                        }
                        Action::Backspace => {
                            self.main_view.handle_search_backspace();
                            self.apply_filters();
                        }
                        Action::Select => {
                            self.apply_filters();
                        }
                        _ => {}
                    }
                } else if self.task_form.is_none() {
                    // Handle calendar navigation when in Reports view and calendar mode
                    if matches!(self.current_view, AppView::Reports) && self.reports_view.is_calendar_mode() {
                        match action {
                            Action::Character('<') => {
                                self.reports_view.navigate_date(crate::ui::views::reports_view::DateNavigation::PrevMonth);
                            }
                            Action::Character('>') => {
                                self.reports_view.navigate_date(crate::ui::views::reports_view::DateNavigation::NextMonth);
                            }
                            Action::Character('t') => {
                                self.reports_view.navigate_date(crate::ui::views::reports_view::DateNavigation::Today);
                            }
                            _ => {}
                        }
                    }
                    
                    // Handle other actions based on current view
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
        // Create header content with title and shortcuts
        let header_content = Line::from(vec![
            Span::styled("LazyTask v0.1", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("                    "),
            Span::styled("[F1]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Help", Style::default().fg(Color::White)),
            Span::raw("    "),
            Span::styled("[F5]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Refresh", Style::default().fg(Color::White)),
            Span::raw("    "),
            Span::styled("[/]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Filter", Style::default().fg(Color::White)),
            Span::raw("    "),
            Span::styled("[r]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Reports", Style::default().fg(Color::White)),
        ]);

        let header = Paragraph::new(header_content)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
            )
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Left);
        
        f.render_widget(header, area);
    }

    fn draw_task_detail(&self, f: &mut Frame, area: Rect) {
        // Show selected task detail in full view
        let selected_task = self.main_view.selected_task();
        let detail_text = if let Some(task) = selected_task {
            format!("Task Detail:\n\n{}", task.description)
        } else {
            "No task selected".to_string()
        };
        let detail = Paragraph::new(detail_text)
            .block(Block::default().title("Task Detail").borders(Borders::ALL));
        f.render_widget(detail, area);
    }

    fn draw_reports(&self, f: &mut Frame, area: Rect) {
        self.reports_view.render(f, area);
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


    fn draw_footer_panel(&self, f: &mut Frame, area: Rect) {
        let help_content = if self.task_form.is_some() {
            Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" Navigate fields  "),
                Span::styled("←→", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw(" Move cursor  "),
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" Save  "),
                Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" Cancel"),
            ])
        } else if self.main_view.is_filter_focused() {
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw(" Next section  "),
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" Navigate  "),
                Span::styled("Space", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" Toggle  "),
                Span::styled("Type", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" Search  "),
                Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" Exit"),
            ])
        } else {
            match self.current_view {
                AppView::TaskList => {
                    Line::from(vec![
                        Span::styled("[a]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw("dd  "),
                        Span::styled("[e]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw("dit  "),
                        Span::styled("[d]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw("one  "),
                        Span::styled("[Del]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw("ete  "),
                        Span::styled("[/]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw("filter  "),
                        Span::styled("[r]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw("eports  "),
                        Span::styled("[q]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::raw("uit"),
                    ])
                }
                AppView::Reports => {
                    if self.reports_view.is_calendar_mode() {
                        Line::from(vec![
                            Span::styled("[←→]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::raw(" day  "),
                            Span::styled("[↑↓]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                            Span::raw(" week  "),
                            Span::styled("[< >]", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                            Span::raw(" month  "),
                            Span::styled("[t]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                            Span::raw("oday  "),
                            Span::styled("[c]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                            Span::raw(" dashboard  "),
                            Span::styled("[ESC]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                            Span::raw(" back"),
                        ])
                    } else {
                        Line::from(vec![
                            Span::styled("[c]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                            Span::raw("alendar  "),
                            Span::styled("[ESC]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                            Span::raw(" back  "),
                            Span::styled("[q]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                            Span::raw("uit"),
                        ])
                    }
                }
                AppView::Help => {
                    Line::from(vec![
                        Span::styled("[ESC]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::raw(" back"),
                    ])
                }
                _ => {
                    Line::from(vec![
                        Span::styled("[ESC]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::raw(" back  "),
                        Span::styled("[q]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::raw("uit"),
                    ])
                }
            }
        };

        let footer_panel = Paragraph::new(help_content)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray))
            )
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center);
        
        f.render_widget(footer_panel, area);
    }

    async fn handle_task_list_action(&mut self, action: Action, taskwarrior: &TaskwarriorIntegration) -> Result<()> {
        match action {
            Action::AddTask => {
                self.task_form = Some(TaskForm::new_task());
            }
            Action::EditTask => {
                if let Some(task) = self.main_view.selected_task() {
                    self.task_form = Some(TaskForm::edit_task(task.clone()));
                }
            }
            Action::DoneTask => {
                if let Some(task) = self.main_view.selected_task() {
                    if let Some(task_id) = task.id {
                        // Find the next task to select after completing this one
                        let current_index = self.main_view.selected_index().unwrap_or(0);
                        let next_task_uuid = if current_index + 1 < self.filtered_tasks.len() {
                            // Select next task
                            Some(self.filtered_tasks[current_index + 1].uuid.clone())
                        } else if current_index > 0 {
                            // Select previous task if we're at the end
                            Some(self.filtered_tasks[current_index - 1].uuid.clone())
                        } else {
                            None // No other tasks available
                        };
                        
                        self.preserve_selection_uuid = next_task_uuid;
                        
                        // Attempt to complete the task with better error handling
                        match taskwarrior.done_task(task_id).await {
                            Ok(_) => {
                                // Successfully completed, reload tasks
                                self.load_tasks(taskwarrior).await?;
                            }
                            Err(e) => {
                                // If completion fails, don't crash - just show the error and continue
                                eprintln!("Failed to complete task {}: {}", task_id, e);
                                // Clear the preserve UUID since operation failed
                                self.preserve_selection_uuid = None;
                            }
                        }
                    }
                }
            }
            Action::DeleteTask => {
                if let Some(task) = self.main_view.selected_task() {
                    if let Some(task_id) = task.id {
                        // Find the next task to select after deleting this one
                        let current_index = self.main_view.selected_index().unwrap_or(0);
                        let next_task_uuid = if current_index + 1 < self.filtered_tasks.len() {
                            // Select next task
                            Some(self.filtered_tasks[current_index + 1].uuid.clone())
                        } else if current_index > 0 {
                            // Select previous task if we're at the end
                            Some(self.filtered_tasks[current_index - 1].uuid.clone())
                        } else {
                            None // No other tasks available
                        };
                        
                        self.preserve_selection_uuid = next_task_uuid;
                        
                        // Attempt to delete the task with better error handling
                        match taskwarrior.delete_task(task_id).await {
                            Ok(_) => {
                                // Successfully deleted, reload tasks
                                self.load_tasks(taskwarrior).await?;
                            }
                            Err(e) => {
                                // If delete fails, don't crash - just show the error and continue
                                eprintln!("Failed to delete task {}: {}", task_id, e);
                                // Clear the preserve UUID since operation failed
                                self.preserve_selection_uuid = None;
                                // Don't propagate the error to avoid crashing the application
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
