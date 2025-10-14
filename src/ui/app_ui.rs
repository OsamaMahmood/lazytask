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
    // Track the task UUID to preserve selection after operations
    preserve_selection_uuid: Option<String>,
    // Track computed filter states
    filter_active: bool,
    filter_overdue: bool,
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
            preserve_selection_uuid: None,
            filter_active: false,
            filter_overdue: false,
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
        // Extract unique projects from pending/active tasks only (matching `task projects`)
        let mut projects: Vec<String> = self.tasks
            .iter()
            .filter(|task| {
                // Only include projects from pending, waiting, or recurring tasks
                matches!(task.status, 
                    crate::data::models::TaskStatus::Pending | 
                    crate::data::models::TaskStatus::Waiting |
                    crate::data::models::TaskStatus::Recurring
                )
            })
            .filter_map(|task| task.project.as_ref())
            .cloned()
            .collect();
        projects.sort();
        projects.dedup();
        self.available_projects = projects.clone();

        // Extract unique tags from pending/active tasks only (matching `task tags`)
        let mut tags: Vec<String> = self.tasks
            .iter()
            .filter(|task| {
                // Only include tags from pending, waiting, or recurring tasks
                matches!(task.status, 
                    crate::data::models::TaskStatus::Pending | 
                    crate::data::models::TaskStatus::Waiting |
                    crate::data::models::TaskStatus::Recurring
                )
            })
            .flat_map(|task| task.tags.iter())
            .cloned()
            .collect();
        tags.sort();
        tags.dedup();
        self.available_tags = tags.clone();

        // Update filter bar widget with current projects and tags
        self.filter_bar_widget.update_available_options(projects, tags);
    }

    fn apply_filters(&mut self) {
        // Apply custom filters based on selections
        self.filtered_tasks = self.tasks
            .iter()
            .filter(|task| self.matches_filters(task))
            .cloned()
            .collect();
        
        // Use preserved selection if available
        let preserve_uuid = self.preserve_selection_uuid.as_deref();
        self.task_list_widget.set_tasks_with_preserved_selection(self.filtered_tasks.clone(), preserve_uuid);
        
        // Clear the preserve UUID after using it
        self.preserve_selection_uuid = None;
    }

    fn matches_filters(&self, task: &Task) -> bool {
        // Status filter (including computed states)
        if !self.selected_statuses.is_empty() || self.filter_active || self.filter_overdue {
            let mut status_matches = false;
            
            // Check basic status matches
            if !self.selected_statuses.is_empty() {
                status_matches = self.selected_statuses.contains(&task.status);
            }
            
            // Check computed state filters
            if self.filter_active && task.is_active() {
                status_matches = true;
            }
            
            if self.filter_overdue && task.is_overdue() {
                status_matches = true;
            }
            
            if !status_matches {
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
                // Only split for filters when in TaskList view
                let available_height = main_chunks[1].height;
                let filter_height = if available_height < 20 {
                    9   // Compact filter area for small screens
                } else if available_height < 30 {
                    12  // Medium filter area for medium screens
                } else {
                    15  // Larger filter area for large screens
                };

                let main_content_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(10),                    // Top area (minimum 10 lines for task list)
                        Constraint::Length(filter_height),     // Responsive filters pane
                    ])
                    .split(main_chunks[1]);

                // Responsive horizontal split based on terminal width
                let terminal_width = size.width;
                let (left_pct, right_pct) = if terminal_width < 100 {
                    (50, 50)  // Equal split for narrow terminals
                } else if terminal_width < 150 {
                    (50, 50)  // Slightly favor detail panel for medium terminals  
                } else {
                    (50, 50)  // More space for detail panel on wide terminals
                };

                let top_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(left_pct),   // Responsive task list
                        Constraint::Percentage(right_pct),  // Responsive task detail
                    ])
                    .split(main_content_chunks[0]);

                // Draw task list on the left
                self.draw_task_list(f, top_chunks[0]);
                // Draw task detail on the right
                self.draw_task_detail_panel(f, top_chunks[1]);
                // Draw filters at the bottom spanning full width
                self.draw_filters_panel(f, main_content_chunks[1]);
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
                            self.preserve_selection_uuid = self.task_list_widget.selected_task_uuid();
                            
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
        let terminal_width = area.width;
        
        // Responsive filter layout based on terminal width
        let filter_chunks = if terminal_width < 120 {
            // Stack filters vertically on narrow screens
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(35), // Status + Project (combined)
                    Constraint::Percentage(35), // Tags + Search (combined)
                    Constraint::Percentage(30), // Additional space
                ])
                .split(area)
        } else {
            // Horizontal layout for wider screens with responsive widths
            let widths = if terminal_width < 160 {
                [20, 30, 25, 25] // Compact layout
            } else {
                [25, 25, 25, 25] // Full layout
            };
            
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(widths[0]), // Status filters
                    Constraint::Percentage(widths[1]), // Project filters 
                    Constraint::Percentage(widths[2]), // Tag filters
                    Constraint::Percentage(widths[3]), // Search filters
                ])
                .split(area)
        };

        if terminal_width < 120 {
            // Narrow screen: combine filters in vertical layout
            let top_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(filter_chunks[0]);
            let bottom_row = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(filter_chunks[1]);
                
            self.draw_status_filters(f, top_row[0]);
            self.draw_project_filters(f, top_row[1]);
            self.draw_tag_filters(f, bottom_row[0]);
            self.draw_search_filter(f, bottom_row[1]);
        } else {
            // Wide screen: horizontal layout
            self.draw_status_filters(f, filter_chunks[0]);
            self.draw_project_filters(f, filter_chunks[1]);
            self.draw_tag_filters(f, filter_chunks[2]);
            self.draw_search_filter(f, filter_chunks[3]);
        }
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
                    1 => self.filter_active,   // Active computed filter
                    2 => self.filter_overdue,  // Overdue computed filter  
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

        // Robust scrolling: calculate viewport accounting for scroll indicators
        let base_visible_items = (area.height as usize).saturating_sub(4).max(1); // Account for borders + header + selection display
        let total_items = self.available_projects.len();
        
        // Reserve space for scroll indicators if we need scrolling
        let needs_scrolling = total_items > base_visible_items;
        let scroll_indicator_space = if needs_scrolling { 2 } else { 0 }; // Reserve 2 lines for ↑ and ↓ indicators
        let max_visible_items = base_visible_items.saturating_sub(scroll_indicator_space).max(1);
        
        let scroll_offset = if total_items <= max_visible_items {
            // All items fit, no scrolling needed
            0
        } else {
            // Calculate scroll offset to keep selected item visible
            let selected_index = self.project_selection_index.min(total_items.saturating_sub(1));
            
            // Ensure selected item is always visible in the viewport
            if selected_index < max_visible_items / 2 {
                // Near the top of list - show from beginning
                0
            } else if selected_index >= total_items - (max_visible_items / 2) {
                // Near the bottom of list - show last page
                total_items.saturating_sub(max_visible_items)
            } else {
                // Middle of list - center the selected item
                selected_index.saturating_sub(max_visible_items / 2)
            }
        };

        let visible_projects: Vec<_> = self.available_projects
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(max_visible_items)
            .collect();

        // Enhanced scroll indicators showing position
        if scroll_offset > 0 {
            project_text.push(Line::from(vec![
                Span::styled(
                    format!("↑ {} more above", scroll_offset),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                ),
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
            
            // Truncate long project names dynamically based on panel width
            let max_chars = (area.width as usize).saturating_sub(6).max(8);
            let display_name = if project.len() > max_chars {
                format!("{}...", &project[..max_chars.saturating_sub(3)])
            } else {
                project.to_string()
            };
            
            project_text.push(Line::from(vec![
                checkbox,
                Span::styled(display_name, text_style),
            ]));
        }

        // Enhanced scroll indicators showing remaining items
        let items_below = self.available_projects.len().saturating_sub(scroll_offset + visible_projects.len());
        if items_below > 0 {
            project_text.push(Line::from(vec![
                Span::styled(
                    format!("↓ {} more below", items_below),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                ),
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

        // Robust scrolling: calculate viewport accounting for scroll indicators
        let base_visible_items = (area.height as usize).saturating_sub(4).max(1); // Account for borders + header + selection display
        let total_items = self.available_tags.len();
        
        // Reserve space for scroll indicators if we need scrolling
        let needs_scrolling = total_items > base_visible_items;
        let scroll_indicator_space = if needs_scrolling { 2 } else { 0 }; // Reserve 2 lines for ↑ and ↓ indicators
        let max_visible_items = base_visible_items.saturating_sub(scroll_indicator_space).max(1);
        
        let scroll_offset = if total_items <= max_visible_items {
            // All items fit, no scrolling needed
            0
        } else {
            // Calculate scroll offset to keep selected item visible
            let selected_index = self.tag_selection_index.min(total_items.saturating_sub(1));
            
            // Ensure selected item is always visible in the viewport
            if selected_index < max_visible_items / 2 {
                // Near the top of list - show from beginning
                0
            } else if selected_index >= total_items - (max_visible_items / 2) {
                // Near the bottom of list - show last page
                total_items.saturating_sub(max_visible_items)
            } else {
                // Middle of list - center the selected item
                selected_index.saturating_sub(max_visible_items / 2)
            }
        };

        let visible_tags: Vec<_> = self.available_tags
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(max_visible_items)
            .collect();

        // Enhanced scroll indicators showing position
        if scroll_offset > 0 {
            tag_text.push(Line::from(vec![
                Span::styled(
                    format!("↑ {} more above", scroll_offset),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                ),
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
            
            // Truncate long tag names dynamically based on panel width
            let max_chars = (area.width as usize).saturating_sub(6).max(6);
            let display_name = if tag.len() > max_chars {
                format!("{}...", &tag[..max_chars.saturating_sub(3)])
            } else {
                tag.to_string()
            };
            
            tag_text.push(Line::from(vec![
                checkbox,
                Span::styled(display_name, text_style),
            ]));
        }

        // Enhanced scroll indicators showing remaining items
        let items_below = self.available_tags.len().saturating_sub(scroll_offset + visible_tags.len());
        if items_below > 0 {
            tag_text.push(Line::from(vec![
                Span::styled(
                    format!("↓ {} more below", items_below),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                ),
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
                match self.status_selection_index {
                    0 => {
                        // Pending status
                        let status = crate::data::models::TaskStatus::Pending;
                        if self.selected_statuses.contains(&status) {
                            self.selected_statuses.retain(|s| s != &status);
                        } else {
                            self.selected_statuses.push(status);
                        }
                    }
                    1 => {
                        // Active (computed filter)
                        self.filter_active = !self.filter_active;
                    }
                    2 => {
                        // Overdue (computed filter)  
                        self.filter_overdue = !self.filter_overdue;
                    }
                    3 => {
                        // Completed status
                        let status = crate::data::models::TaskStatus::Completed;
                        if self.selected_statuses.contains(&status) {
                            self.selected_statuses.retain(|s| s != &status);
                        } else {
                            self.selected_statuses.push(status);
                        }
                    }
                    _ => {}
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
                        // Find the next task to select after completing this one
                        let current_index = self.task_list_widget.state.selected().unwrap_or(0);
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
                if let Some(task) = self.task_list_widget.selected_task() {
                    if let Some(task_id) = task.id {
                        // Find the next task to select after deleting this one
                        let current_index = self.task_list_widget.state.selected().unwrap_or(0);
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
