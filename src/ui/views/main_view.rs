// Primary task list view with detail panel and filters
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::data::models::{Task, TaskStatus};
use crate::ui::components::filter_bar::FilterBarWidget;
use crate::ui::components::task_detail::TaskDetailWidget;
use crate::ui::components::task_list::TaskListWidget;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterSection {
    Status,
    Project,
    Tags,
    Search,
}

pub struct MainView {
    task_list_widget: TaskListWidget,
    task_detail_widget: TaskDetailWidget,
    filter_bar_widget: FilterBarWidget,
    filter_focused: bool,
    active_filter_section: FilterSection,
    status_selection_index: usize,
    project_selection_index: usize,
    tag_selection_index: usize,
    search_text: String,
    available_projects: Vec<String>,
    available_tags: Vec<String>,
    selected_statuses: Vec<TaskStatus>,
    selected_projects: Vec<String>,
    selected_tags: Vec<String>,
    filter_active: bool,
    filter_overdue: bool,
}

impl MainView {
    pub fn new() -> Self {
        MainView {
            task_list_widget: TaskListWidget::new(),
            task_detail_widget: TaskDetailWidget::new(),
            filter_bar_widget: FilterBarWidget::new(),
            filter_focused: false,
            active_filter_section: FilterSection::Status,
            status_selection_index: 0,
            project_selection_index: 0,
            tag_selection_index: 0,
            search_text: String::new(),
            available_projects: Vec::new(),
            available_tags: Vec::new(),
            selected_statuses: vec![TaskStatus::Pending],
            selected_projects: Vec::new(),
            selected_tags: Vec::new(),
            filter_active: false,
            filter_overdue: false,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, terminal_width: u16) {
        let available_height = area.height;
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
            .split(area);

        // Responsive horizontal split based on terminal width
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
        self.task_list_widget.render(f, top_chunks[0]);
        
        // Draw task detail on the right
        let selected_task = self.task_list_widget.selected_task();
        self.task_detail_widget.render(f, top_chunks[1], selected_task);
        
        // Draw filters at the bottom spanning full width
        self.draw_filters_panel(f, main_content_chunks[1], terminal_width);
    }

    pub fn update_available_filters(&mut self, tasks: &[Task]) {
        // Extract unique projects from pending/active tasks only
        let mut projects: Vec<String> = tasks
            .iter()
            .filter(|task| {
                matches!(task.status, 
                    TaskStatus::Pending | 
                    TaskStatus::Waiting |
                    TaskStatus::Recurring
                )
            })
            .filter_map(|task| task.project.as_ref())
            .cloned()
            .collect();
        projects.sort();
        projects.dedup();
        self.available_projects = projects.clone();

        // Extract unique tags from pending/active tasks only
        let mut tags: Vec<String> = tasks
            .iter()
            .filter(|task| {
                matches!(task.status, 
                    TaskStatus::Pending | 
                    TaskStatus::Waiting |
                    TaskStatus::Recurring
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

    pub fn set_tasks_with_preserved_selection(&mut self, tasks: Vec<Task>, preserve_uuid: Option<&str>) {
        self.task_list_widget.set_tasks_with_preserved_selection(tasks, preserve_uuid);
    }

    pub fn matches_filters(&self, task: &Task) -> bool {
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

    // Navigation methods
    pub fn next_task(&mut self) {
        self.task_list_widget.next();
    }

    pub fn previous_task(&mut self) {
        self.task_list_widget.previous();
    }

    pub fn selected_task(&self) -> Option<&Task> {
        self.task_list_widget.selected_task()
    }

    pub fn selected_task_uuid(&self) -> Option<String> {
        self.task_list_widget.selected_task_uuid()
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.task_list_widget.state.selected()
    }

    // Filter management
    pub fn is_filter_focused(&self) -> bool {
        self.filter_focused
    }

    pub fn toggle_filter_focus(&mut self) {
        self.filter_focused = !self.filter_focused;
        if self.filter_focused {
            self.filter_bar_widget.is_visible = true;
        }
    }

    pub fn exit_filter_mode(&mut self) {
        self.filter_focused = false;
        self.filter_bar_widget.is_visible = false;
    }

    pub fn next_filter_section(&mut self) {
        self.active_filter_section = match self.active_filter_section {
            FilterSection::Status => FilterSection::Project,
            FilterSection::Project => FilterSection::Tags,
            FilterSection::Tags => FilterSection::Search,
            FilterSection::Search => FilterSection::Status,
        };
    }

    pub fn handle_filter_navigation_up(&mut self) {
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

    pub fn handle_filter_navigation_down(&mut self) {
        match self.active_filter_section {
            FilterSection::Status => {
                let max_status = 4; // Pending, Active, Overdue, Completed, Deleted (0-4)
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

    pub fn toggle_current_selection(&mut self) {
        match self.active_filter_section {
            FilterSection::Status => {
                match self.status_selection_index {
                    0 => {
                        // Pending status
                        let status = TaskStatus::Pending;
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
                        let status = TaskStatus::Completed;
                        if self.selected_statuses.contains(&status) {
                            self.selected_statuses.retain(|s| s != &status);
                        } else {
                            self.selected_statuses.push(status);
                        }
                    }
                    4 => {
                        // Deleted status
                        let status = TaskStatus::Deleted;
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

    pub fn handle_search_character(&mut self, c: char) {
        if self.active_filter_section == FilterSection::Search {
            self.search_text.push(c);
        }
    }

    pub fn handle_search_backspace(&mut self) {
        if self.active_filter_section == FilterSection::Search {
            self.search_text.pop();
        }
    }

    fn draw_filters_panel(&mut self, f: &mut Frame, area: Rect, terminal_width: u16) {
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
            ("Pending", TaskStatus::Pending),
            ("Active", TaskStatus::Pending),
            ("Overdue", TaskStatus::Pending),
            ("Completed", TaskStatus::Completed),
            ("Deleted", TaskStatus::Deleted),
        ];
        
        let status_text: Vec<Line> = statuses
            .iter()
            .enumerate()
            .map(|(i, (name, _status))| {
                let is_selected = match i {
                    0 => self.selected_statuses.contains(&TaskStatus::Pending),
                    1 => self.filter_active,
                    2 => self.filter_overdue,
                    3 => self.selected_statuses.contains(&TaskStatus::Completed),
                    4 => self.selected_statuses.contains(&TaskStatus::Deleted),
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

        let base_visible_items = (area.height as usize).saturating_sub(4).max(1);
        let total_items = self.available_projects.len();
        
        let needs_scrolling = total_items > base_visible_items;
        let scroll_indicator_space = if needs_scrolling { 2 } else { 0 };
        let max_visible_items = base_visible_items.saturating_sub(scroll_indicator_space).max(1);
        
        let scroll_offset = if total_items <= max_visible_items {
            0
        } else {
            let selected_index = self.project_selection_index.min(total_items.saturating_sub(1));
            
            if selected_index < max_visible_items / 2 {
                0
            } else if selected_index >= total_items - (max_visible_items / 2) {
                total_items.saturating_sub(max_visible_items)
            } else {
                selected_index.saturating_sub(max_visible_items / 2)
            }
        };

        let visible_projects: Vec<_> = self.available_projects
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(max_visible_items)
            .collect();

        if scroll_offset > 0 {
            project_text.push(Line::from(vec![
                Span::styled(
                    format!("↑ {} more above", scroll_offset),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                ),
            ]));
        }

        for (original_i, project) in visible_projects.iter() {
            let is_selected = self.selected_projects.contains(project);
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

        let base_visible_items = (area.height as usize).saturating_sub(4).max(1);
        let total_items = self.available_tags.len();
        
        let needs_scrolling = total_items > base_visible_items;
        let scroll_indicator_space = if needs_scrolling { 2 } else { 0 };
        let max_visible_items = base_visible_items.saturating_sub(scroll_indicator_space).max(1);
        
        let scroll_offset = if total_items <= max_visible_items {
            0
        } else {
            let selected_index = self.tag_selection_index.min(total_items.saturating_sub(1));
            
            if selected_index < max_visible_items / 2 {
                0
            } else if selected_index >= total_items - (max_visible_items / 2) {
                total_items.saturating_sub(max_visible_items)
            } else {
                selected_index.saturating_sub(max_visible_items / 2)
            }
        };

        let visible_tags: Vec<_> = self.available_tags
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(max_visible_items)
            .collect();

        if scroll_offset > 0 {
            tag_text.push(Line::from(vec![
                Span::styled(
                    format!("↑ {} more above", scroll_offset),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                ),
            ]));
        }

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
                        "_".to_string()
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
}
