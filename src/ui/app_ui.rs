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
use crate::ui::components::filter_bar::FilterBarWidget;
use crate::ui::components::task_detail::TaskDetailWidget;
use crate::ui::components::task_form::{TaskForm, TaskFormResult};
use crate::ui::components::task_list::TaskListWidget;
use crate::ui::views::reports_view::ReportsView;

pub enum AppView {
    TaskList,
    TaskDetail,
    Reports,
    Settings,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterSection {
    Status,
    Project,
    Tags,
    Search,
}

pub struct AppUI {
    config: Config,
    current_view: AppView,
    show_help_bar: bool,
    task_list_widget: TaskListWidget,
    task_detail_widget: TaskDetailWidget,
    filter_bar_widget: FilterBarWidget,
    reports_view: ReportsView,
    tasks: Vec<Task>,
    filtered_tasks: Vec<Task>,
    task_form: Option<TaskForm>,
    filter_focused: bool,
    active_filter_section: FilterSection,
    status_selection_index: usize,
    project_selection_index: usize,
    tag_selection_index: usize,
    search_text: String,
    available_projects: Vec<String>,
    available_tags: Vec<String>,
    selected_statuses: Vec<crate::data::models::TaskStatus>,
    selected_projects: Vec<String>,
    selected_tags: Vec<String>,
}

impl AppUI {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(AppUI {
            config: config.clone(),
            current_view: AppView::TaskList,
            show_help_bar: config.ui.show_help_bar,
            task_list_widget: TaskListWidget::new(),
            task_detail_widget: TaskDetailWidget::new(),
            filter_bar_widget: FilterBarWidget::new(),
            reports_view: ReportsView::new(),
            tasks: Vec::new(),
            filtered_tasks: Vec::new(),
            task_form: None,
            filter_focused: false,
            active_filter_section: FilterSection::Status,
            status_selection_index: 0,
            project_selection_index: 0,
            tag_selection_index: 0,
            search_text: String::new(),
            available_projects: Vec::new(),
            available_tags: Vec::new(),
            selected_statuses: vec![crate::data::models::TaskStatus::Pending],
            selected_projects: Vec::new(),
            selected_tags: Vec::new(),
        })
    }

    pub async fn load_tasks(&mut self, taskwarrior: &TaskwarriorIntegration) -> Result<()> {
        // Load all tasks (not just pending) and sort by entry date (newest first)
        let mut tasks = taskwarrior.list_tasks(None).await?;
        tasks.sort_by(|a, b| b.entry.cmp(&a.entry)); // Newest first
        self.tasks = tasks.clone();
        
        // Extract available projects and tags from tasks
        self.update_available_filters();
        
        // Update reports view with all tasks
        self.reports_view.update_tasks(tasks);
        
        self.apply_filters();
        Ok(())
    }

    fn update_available_filters(&mut self) {
        // Extract unique projects
        let mut projects: Vec<String> = self.tasks
            .iter()
            .filter_map(|task| task.project.as_ref())
            .cloned()
            .collect();
        projects.sort();
        projects.dedup();
        self.available_projects = projects;

        // Extract unique tags
        let mut tags: Vec<String> = self.tasks
            .iter()
            .flat_map(|task| task.tags.iter())
            .cloned()
            .collect();
        tags.sort();
        tags.dedup();
        self.available_tags = tags;
    }

    fn apply_filters(&mut self) {
        // Apply custom filters based on selections
        self.filtered_tasks = self.tasks
            .iter()
            .filter(|task| self.matches_filters(task))
            .cloned()
            .collect();
        self.task_list_widget.set_tasks(self.filtered_tasks.clone());
    }

    fn matches_filters(&self, task: &Task) -> bool {
        // Status filter
        if !self.selected_statuses.is_empty() {
            if !self.selected_statuses.contains(&task.status) {
                return false;
            }
        }

        // Project filter
        if !self.selected_projects.is_empty() {
            match &task.project {
                Some(project) => {
                    if !self.selected_projects.contains(project) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Tags filter
        if !self.selected_tags.is_empty() {
            let has_selected_tag = self.selected_tags
                .iter()
                .any(|selected_tag| task.tags.contains(selected_tag));
            if !has_selected_tag {
                return false;
            }
        }

        // Search filter
        if !self.search_text.is_empty() {
            let search_text = self.search_text.to_lowercase();
            let matches_description = task.description.to_lowercase().contains(&search_text);
            let matches_project = task.project.as_ref()
                .map(|p| p.to_lowercase().contains(&search_text))
                .unwrap_or(false);
            let matches_tags = task.tags.iter()
                .any(|tag| tag.to_lowercase().contains(&search_text));
            
            if !matches_description && !matches_project && !matches_tags {
                return false;
            }
        }

        true
    }

    pub fn has_active_form(&self) -> bool {
        self.task_form.is_some() || self.filter_focused
    }

    fn task_to_attributes(task: &Task) -> Vec<(String, String)> {
        let mut attributes = Vec::new();

        // Add description (this was missing!)
        attributes.push(("description".to_string(), task.description.clone()));

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
        
        // Create modern dashboard layout with split panes
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Header with borders
                Constraint::Min(10),        // Content area (task list + filters)
                Constraint::Length(3),      // Footer panel with boundaries
            ])
            .split(size);

        // Draw header
        self.draw_header(f, main_chunks[0]);

        // Create 3-pane layout: task list + task detail (top), filters (bottom)
        let main_content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(12),        // Top area (task list + task detail)
                Constraint::Length(8),      // Filters pane (bottom)
            ])
            .split(main_chunks[1]);

        // Split top area into task list (left) and task detail (right)
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45), // Task list (left side)
                Constraint::Percentage(55), // Task detail (right side)
            ])
            .split(main_content_chunks[0]);

        // Draw main content based on current view
                match self.current_view {
            AppView::TaskList => {
                // Draw task list on the left
                self.draw_task_list(f, top_chunks[0]);
                // Draw task detail on the right
                self.draw_task_detail_panel(f, top_chunks[1]);
                // Draw filters at the bottom spanning full width
                self.draw_filters_panel(f, main_content_chunks[1]);
            }
            AppView::TaskDetail => self.draw_task_detail(f, main_content_chunks[0]),
            AppView::Reports => self.draw_reports(f, main_content_chunks[0]),
            AppView::Settings => self.draw_settings(f, main_content_chunks[0]),
            AppView::Help => self.draw_help(f, main_content_chunks[0]),
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
                } else if self.filter_focused {
                    // Single ESC to exit filter mode
                    self.filter_focused = false;
                    self.filter_bar_widget.is_visible = false;
                    self.apply_filters(); // Apply filters when exiting
                } else {
                    self.current_view = AppView::TaskList;
                }
            }
            Action::MoveUp => {
                if self.filter_focused {
                    self.handle_filter_navigation_up();
                } else if self.task_form.is_none() && matches!(self.current_view, AppView::TaskList) {
                    self.task_list_widget.previous();
                }
            }
            Action::MoveDown => {
                if self.filter_focused {
                    self.handle_filter_navigation_down();
                } else if self.task_form.is_none() && matches!(self.current_view, AppView::TaskList) {
                    self.task_list_widget.next();
                }
            }
            Action::Refresh => {
                self.load_tasks(taskwarrior).await?;
            }
            Action::Filter => {
                // Toggle filter focus instead of just visibility
                self.filter_focused = !self.filter_focused;
                if self.filter_focused {
                    // Entering filter mode
                    self.filter_bar_widget.is_visible = true;
                } else {
                    // Exiting filter mode - apply filters
                    self.apply_filters();
                }
            }
            Action::Tab => {
                if self.filter_focused {
                    self.next_filter_section();
                }
            }
            _ => {
                // Handle filter actions if filters are focused
                if self.filter_focused {
                    // Don't pass navigation actions to handle_filter_action - they're handled above
                    match action {
                        Action::MoveUp | Action::MoveDown => {
                            // Already handled above, do nothing
                        }
                        _ => {
                            self.handle_filter_action(action).await?;
                        }
                    }
                } else if self.task_form.is_none() {
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

    fn draw_split_view(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(45), // Task list - reduced for more detail space
                Constraint::Percentage(55), // Task detail - increased for comprehensive view
            ])
            .split(area);

        // Draw task list on the left
        self.draw_task_list(f, chunks[0]);

        // Draw task detail on the right
        self.draw_task_detail_panel(f, chunks[1]);
    }

    fn draw_task_list(&mut self, f: &mut Frame, area: Rect) {
        self.task_list_widget.render(f, area);
    }

    fn draw_task_detail_panel(&self, f: &mut Frame, area: Rect) {
        let selected_task = self.task_list_widget.selected_task();
        self.task_detail_widget.render(f, area, selected_task);
    }

    fn draw_task_detail(&self, f: &mut Frame, area: Rect) {
        // For now, show the selected task from the task list widget
        // In the future, this could show a specific task based on navigation state
        let selected_task = self.task_list_widget.selected_task();
        self.task_detail_widget.render(f, area, selected_task);
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

    fn draw_filters_panel(&mut self, f: &mut Frame, area: Rect) {
        // Uniform layout: All panels get equal 25% width for perfect balance
        let filter_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Status filters
                Constraint::Percentage(25), // Project filters 
                Constraint::Percentage(25), // Tag filters
                Constraint::Percentage(25), // Search filters
            ])
            .split(area);

        // Status filters section
        self.draw_status_filters(f, filter_chunks[0]);
        
        // Project filters section  
        self.draw_project_filters(f, filter_chunks[1]);
        
        // Tag filters section
        self.draw_tag_filters(f, filter_chunks[2]);
        
        // Generic search section
        self.draw_search_filter(f, filter_chunks[3]);
    }
    
    fn draw_status_filters(&self, f: &mut Frame, area: Rect) {
        let statuses = [
            ("Pending", crate::data::models::TaskStatus::Pending),
            ("Active", crate::data::models::TaskStatus::Pending), // We'll handle active differently
            ("Overdue", crate::data::models::TaskStatus::Pending), // We'll handle overdue differently
            ("Completed", crate::data::models::TaskStatus::Completed),
        ];
        
        let status_text: Vec<Line> = statuses
            .iter()
            .enumerate()
            .map(|(i, (name, _status))| {
                let is_selected = match i {
                    0 => self.selected_statuses.contains(&crate::data::models::TaskStatus::Pending),
                    1 => false, // Active - implement later
                    2 => false, // Overdue - implement later  
                    3 => self.selected_statuses.contains(&crate::data::models::TaskStatus::Completed),
                    _ => false,
                };
                
                let is_highlighted = self.active_filter_section == FilterSection::Status 
                    && self.status_selection_index == i;
                
                let checkbox = if is_selected {
                    Span::styled("[✓] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("[ ] ", Style::default().fg(Color::Gray))
                };
                
                let text_style = if is_highlighted {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                
                Line::from(vec![
                    checkbox,
                    Span::styled(*name, text_style),
                ])
            })
            .collect();

        let border_color = if self.filter_focused && self.active_filter_section == FilterSection::Status {
            Color::Yellow
        } else if self.filter_focused {
            Color::DarkGray
        } else {
            Color::Cyan
        };
        
        let status_panel = Paragraph::new(status_text)
            .block(Block::default()
                .title("Status")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::White));
        
        f.render_widget(status_panel, area);
    }
    
    fn draw_project_filters(&self, f: &mut Frame, area: Rect) {
        let mut project_text = vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if self.selected_projects.is_empty() {
                        "None".to_string()
                    } else {
                        // Truncate long selection list
                        let selection = self.selected_projects.join(", ");
                        if selection.len() > 20 {
                            format!("{}...", &selection[..17])
                        } else {
                            selection
                        }
                    },
                    Style::default().fg(Color::Green)
                ),
            ]),
            Line::from(""),
        ];

        // Simplified scrolling: show all projects if they fit, otherwise show from selected item
        let max_visible_items = (area.height as usize).saturating_sub(4).max(1); // Account for borders + header
        let scroll_offset = if self.available_projects.len() <= max_visible_items {
            // All projects fit, no scrolling needed
            0
        } else if self.active_filter_section == FilterSection::Project {
            // Center the selected item in the visible area
            if self.project_selection_index < max_visible_items / 2 {
                0
            } else if self.project_selection_index >= self.available_projects.len() - max_visible_items / 2 {
                self.available_projects.len().saturating_sub(max_visible_items)
            } else {
                self.project_selection_index.saturating_sub(max_visible_items / 2)
            }
        } else {
            0
        };

        let visible_projects: Vec<_> = self.available_projects
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(max_visible_items)
            .collect();

        // Show scroll indicators if needed
        if scroll_offset > 0 {
            project_text.push(Line::from(vec![
                Span::styled("↑ More above...", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
        }

        // Show visible projects with highlighting
        for (original_i, project) in visible_projects.iter() {
            let is_selected = self.selected_projects.contains(project);
            // FIX: Compare with original index, not visual index
            let is_highlighted = self.active_filter_section == FilterSection::Project 
                && self.project_selection_index == *original_i;
            
            let checkbox = if is_selected {
                Span::styled("[✓] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("[ ] ", Style::default().fg(Color::Gray))
            };
            
            let text_style = if is_highlighted {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            // Truncate long project names
            let display_name = if project.len() > 15 {
                format!("{}...", &project[..12])
            } else {
                project.to_string()
            };
            
            project_text.push(Line::from(vec![
                checkbox,
                Span::styled(display_name, text_style),
            ]));
        }

        // Show scroll indicator for more below
        if scroll_offset + visible_projects.len() < self.available_projects.len() {
            project_text.push(Line::from(vec![
                Span::styled("↓ More below...", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
        }

        let border_color = if self.filter_focused && self.active_filter_section == FilterSection::Project {
            Color::Yellow
        } else if self.filter_focused {
            Color::DarkGray
        } else {
            Color::Cyan
        };

        let project_panel = Paragraph::new(project_text)
            .block(Block::default()
                .title("Project")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::White));
        
        f.render_widget(project_panel, area);
    }
    
    fn draw_tag_filters(&self, f: &mut Frame, area: Rect) {
        let mut tag_text = vec![
            Line::from(vec![
                Span::styled("Selected: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if self.selected_tags.is_empty() {
                        "None".to_string()
                    } else {
                        // Truncate long selection list
                        let selection = format!("+{}", self.selected_tags.join(" +"));
                        if selection.len() > 20 {
                            format!("{}...", &selection[..17])
                        } else {
                            selection
                        }
                    },
                    Style::default().fg(Color::Green)
                ),
            ]),
            Line::from(""),
        ];

        // Simplified scrolling: show all tags if they fit, otherwise show from selected item
        let max_visible_items = (area.height as usize).saturating_sub(4).max(1); // Account for borders + header
        let scroll_offset = if self.available_tags.len() <= max_visible_items {
            // All tags fit, no scrolling needed
            0
        } else if self.active_filter_section == FilterSection::Tags {
            // Center the selected item in the visible area
            if self.tag_selection_index < max_visible_items / 2 {
                0
            } else if self.tag_selection_index >= self.available_tags.len() - max_visible_items / 2 {
                self.available_tags.len().saturating_sub(max_visible_items)
            } else {
                self.tag_selection_index.saturating_sub(max_visible_items / 2)
            }
        } else {
            0
        };

        let visible_tags: Vec<_> = self.available_tags
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(max_visible_items)
            .collect();

        // Show scroll indicators if needed
        if scroll_offset > 0 {
            tag_text.push(Line::from(vec![
                Span::styled("↑ More above...", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
        }

        // Show visible tags with highlighting
        for (original_i, tag) in visible_tags.iter() {
            let is_selected = self.selected_tags.contains(tag);
            let is_highlighted = self.active_filter_section == FilterSection::Tags 
                && self.tag_selection_index == *original_i;
            
            let checkbox = if is_selected {
                Span::styled("[✓] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("[ ] ", Style::default().fg(Color::Gray))
            };
            
            let text_style = if is_highlighted {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            // Truncate long tag names
            let display_name = if tag.len() > 15 {
                format!("{}...", &tag[..12])
            } else {
                tag.to_string()
            };
            
            tag_text.push(Line::from(vec![
                checkbox,
                Span::styled(display_name, text_style),
            ]));
        }

        // Show scroll indicator for more below
        if scroll_offset + visible_tags.len() < self.available_tags.len() {
            tag_text.push(Line::from(vec![
                Span::styled("↓ More below...", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
        }

        let border_color = if self.filter_focused && self.active_filter_section == FilterSection::Tags {
            Color::Yellow
        } else if self.filter_focused {
            Color::DarkGray
        } else {
            Color::Cyan
        };

        let tag_panel = Paragraph::new(tag_text)
            .block(Block::default()
                .title("Tags")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::White));
        
        f.render_widget(tag_panel, area);
    }

    fn draw_search_filter(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_filter_section == FilterSection::Search;
        
        let mut search_text = vec![
            Line::from(vec![
                Span::styled("Search: ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    if self.search_text.is_empty() && is_active {
                        "_".to_string()  // Show cursor when active
                    } else {
                        self.search_text.clone()
                    },
                    if is_active {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Green)
                    }
                ),
            ]),
            Line::from(""),
        ];

        if is_active {
            search_text.push(Line::from(vec![
                Span::styled("Type to search", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
        } else {
            search_text.extend(vec![
                Line::from("Searches in:"),
                Line::from("• Description"),
                Line::from("• Project"),
                Line::from("• Tags"),
            ]);
        }

        let border_color = if self.filter_focused && self.active_filter_section == FilterSection::Search {
            Color::Yellow
        } else if self.filter_focused {
            Color::DarkGray
        } else {
            Color::Cyan
        };

        let search_panel = Paragraph::new(search_text)
            .block(Block::default()
                .title("Search")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::White));
        
        f.render_widget(search_panel, area);
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
        } else if self.filter_focused {
            Line::from(vec![
                Span::styled("Tab", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw(" Next section  "),
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" Navigate  "),
                Span::styled("Space", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" Toggle  "),
                if self.active_filter_section == FilterSection::Search {
                    Span::styled("Type", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                } else {
                    Span::styled("Type", Style::default().fg(Color::DarkGray))
                },
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

    async fn handle_filter_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Space => {
                // Toggle current selection in active section
                self.toggle_current_selection();
                self.apply_filters();
            }
            Action::Character(c) => {
                // Only handle characters in Search section
                if self.active_filter_section == FilterSection::Search {
                    self.search_text.push(c);
                    self.apply_filters();
                }
            }
            Action::Backspace => {
                // Only handle backspace in Search section
                if self.active_filter_section == FilterSection::Search {
                    self.search_text.pop();
                    self.apply_filters();
                }
            }
            Action::Select => {
                // Apply filters and stay in filter mode
                self.apply_filters();
            }
            _ => {
                // Other actions are ignored in filter mode
            }
        }
        Ok(())
    }

    fn next_filter_section(&mut self) {
        self.active_filter_section = match self.active_filter_section {
            FilterSection::Status => FilterSection::Project,
            FilterSection::Project => FilterSection::Tags,
            FilterSection::Tags => FilterSection::Search,
            FilterSection::Search => FilterSection::Status,
        };
    }

    fn handle_filter_navigation_up(&mut self) {
        match self.active_filter_section {
            FilterSection::Status => {
                if self.status_selection_index > 0 {
                    self.status_selection_index -= 1;
                }
            }
            FilterSection::Project => {
                if self.project_selection_index > 0 {
                    self.project_selection_index -= 1;
                }
            }
            FilterSection::Tags => {
                if self.tag_selection_index > 0 {
                    self.tag_selection_index -= 1;
                }
            }
            FilterSection::Search => {
                // No navigation in search
            }
        }
    }

    fn handle_filter_navigation_down(&mut self) {
        match self.active_filter_section {
            FilterSection::Status => {
                let max_status = 3; // Pending, Active, Overdue, Completed (0-3)
                if self.status_selection_index < max_status {
                    self.status_selection_index += 1;
                }
            }
            FilterSection::Project => {
                if !self.available_projects.is_empty() && self.project_selection_index < self.available_projects.len() - 1 {
                    self.project_selection_index += 1;
                }
            }
            FilterSection::Tags => {
                if !self.available_tags.is_empty() && self.tag_selection_index < self.available_tags.len() - 1 {
                    self.tag_selection_index += 1;
                }
            }
            FilterSection::Search => {
                // No navigation in search
            }
        }
    }

    fn toggle_current_selection(&mut self) {
        match self.active_filter_section {
            FilterSection::Status => {
                let status = match self.status_selection_index {
                    0 => crate::data::models::TaskStatus::Pending,
                    1 => crate::data::models::TaskStatus::Pending, // Representing Active
                    2 => crate::data::models::TaskStatus::Pending, // Representing Overdue  
                    3 => crate::data::models::TaskStatus::Completed,
                    _ => crate::data::models::TaskStatus::Pending,
                };
                
                if self.selected_statuses.contains(&status) {
                    self.selected_statuses.retain(|s| s != &status);
                } else {
                    self.selected_statuses.push(status);
                }
            }
            FilterSection::Project => {
                if let Some(project) = self.available_projects.get(self.project_selection_index) {
                    if self.selected_projects.contains(project) {
                        self.selected_projects.retain(|p| p != project);
                    } else {
                        self.selected_projects.push(project.clone());
                    }
                }
            }
            FilterSection::Tags => {
                if let Some(tag) = self.available_tags.get(self.tag_selection_index) {
                    if self.selected_tags.contains(tag) {
                        self.selected_tags.retain(|t| t != tag);
                    } else {
                        self.selected_tags.push(tag.clone());
                    }
                }
            }
            FilterSection::Search => {
                // No toggle in search
            }
        }
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
