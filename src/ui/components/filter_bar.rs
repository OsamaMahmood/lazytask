// Interactive filter builder component

use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::data::filters::TaskFilter;
use crate::data::models::{Priority, TaskStatus};
use crate::handlers::input::Action;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusFilterType {
    All,
    Pending,
    Completed,
    Active,    // Special: pending + started
    Overdue,   // Special: pending + past due
    Waiting,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterField {
    Project,
    Priority,
    Status,
    Tags,
    Description,
}

pub struct FilterBarWidget {
    pub filter: TaskFilter,
    pub active_field: FilterField,
    pub is_editing: bool,
    pub project_input: String,
    pub tags_input: String,
    pub description_input: String,
    pub status_filter_type: StatusFilterType,
    pub is_visible: bool,
    pub available_projects: Vec<String>,
    pub available_tags: Vec<String>,
}

impl FilterBarWidget {
    pub fn new() -> Self {
        FilterBarWidget {
            filter: TaskFilter::default(),
            active_field: FilterField::Status,
            is_editing: false,
            project_input: String::new(),
            tags_input: String::new(),
            description_input: String::new(),
            status_filter_type: StatusFilterType::Pending, // Default to pending (matches TaskFilter::default())
            is_visible: false,
            available_projects: Vec::new(),
            available_tags: Vec::new(),
        }
    }

    pub fn update_available_options(&mut self, projects: Vec<String>, tags: Vec<String>) {
        self.available_projects = projects;
        self.available_tags = tags;
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
        if self.is_visible {
            self.is_editing = true;
        }
    }

    pub fn handle_input(&mut self, action: Action) -> Result<bool> {
        if !self.is_visible {
            return Ok(false);
        }

        match action {
            Action::Back => {
                if self.is_editing {
                    self.is_editing = false;
                } else {
                    self.is_visible = false;
                }
                return Ok(true);
            }
            Action::Select => {
                if self.is_editing {
                    self.apply_current_field();
                    self.is_editing = false;
                } else {
                    return Ok(false); // Apply filters
                }
                return Ok(true);
            }
            Action::MoveDown => {
                if !self.is_editing {
                    self.next_field();
                }
                return Ok(true);
            }
            Action::MoveUp => {
                if !self.is_editing {
                    self.previous_field();
                }
                return Ok(true);
            }
            Action::MoveRight => {
                if !self.is_editing {
                    self.is_editing = true;
                }
                return Ok(true);
            }
            Action::Character(c) => {
                if c == 'C' && !self.is_editing {
                    // Clear all filters when 'C' is pressed outside editing mode
                    self.clear_filters();
                } else if self.is_editing {
                    self.handle_character_input(c);
                }
                return Ok(true);
            }
            Action::Space => {
                if matches!(self.active_field, FilterField::Status) {
                    // Space cycles through status filter types
                    self.cycle_status_filter();
                    self.apply_status_filter();
                    return Ok(true);
                }
                return Ok(false);
            }
            Action::Backspace => {
                if self.is_editing {
                    self.handle_backspace();
                }
                return Ok(true);
            }
            _ => return Ok(false),
        }
    }

    fn next_field(&mut self) {
        self.active_field = match self.active_field {
            FilterField::Status => FilterField::Priority,
            FilterField::Priority => FilterField::Project,
            FilterField::Project => FilterField::Tags,
            FilterField::Tags => FilterField::Description,
            FilterField::Description => FilterField::Status,
        };
    }

    fn previous_field(&mut self) {
        self.active_field = match self.active_field {
            FilterField::Status => FilterField::Description,
            FilterField::Priority => FilterField::Status,
            FilterField::Project => FilterField::Priority,
            FilterField::Tags => FilterField::Project,
            FilterField::Description => FilterField::Tags,
        };
    }

    fn handle_character_input(&mut self, c: char) {
        match self.active_field {
            FilterField::Project => self.project_input.push(c),
            FilterField::Tags => self.tags_input.push(c),
            FilterField::Description => self.description_input.push(c),
            FilterField::Priority => {
                match c.to_ascii_uppercase() {
                    'H' => self.filter.priority = Some(Priority::High),
                    'M' => self.filter.priority = Some(Priority::Medium),
                    'L' => self.filter.priority = Some(Priority::Low),
                    'N' => self.filter.priority = None,
                    _ => {}
                }
            }
            FilterField::Status => {
                match c.to_ascii_lowercase() {
                    'a' => {
                        if self.status_filter_type == StatusFilterType::All {
                            self.status_filter_type = StatusFilterType::Active;
                        } else {
                            self.status_filter_type = StatusFilterType::All;
                        }
                    }
                    'p' => self.status_filter_type = StatusFilterType::Pending,
                    'c' => self.status_filter_type = StatusFilterType::Completed,
                    'd' => self.status_filter_type = StatusFilterType::Deleted,
                    'w' => self.status_filter_type = StatusFilterType::Waiting,
                    'o' => self.status_filter_type = StatusFilterType::Overdue,
                    _ => {}
                }
                self.apply_status_filter();
            }
        }
    }

    fn handle_backspace(&mut self) {
        match self.active_field {
            FilterField::Project => { self.project_input.pop(); }
            FilterField::Tags => { self.tags_input.pop(); }
            FilterField::Description => { self.description_input.pop(); }
            FilterField::Priority => self.filter.priority = None,
            FilterField::Status => {
                self.status_filter_type = StatusFilterType::All;
                self.apply_status_filter();
            }
        }
    }

    fn cycle_status_filter(&mut self) {
        self.status_filter_type = match self.status_filter_type {
            StatusFilterType::All => StatusFilterType::Pending,
            StatusFilterType::Pending => StatusFilterType::Active,
            StatusFilterType::Active => StatusFilterType::Overdue,
            StatusFilterType::Overdue => StatusFilterType::Completed,
            StatusFilterType::Completed => StatusFilterType::Waiting,
            StatusFilterType::Waiting => StatusFilterType::Deleted,
            StatusFilterType::Deleted => StatusFilterType::All,
        };
    }

    fn apply_status_filter(&mut self) {
        // Reset all status-related filters first
        self.filter.status = None;
        self.filter.is_active = None;
        self.filter.is_overdue = None;

        // Apply the selected status filter type
        match self.status_filter_type {
            StatusFilterType::All => {
                // No filters - show all tasks
            }
            StatusFilterType::Pending => {
                self.filter.status = Some(TaskStatus::Pending);
            }
            StatusFilterType::Completed => {
                self.filter.status = Some(TaskStatus::Completed);
            }
            StatusFilterType::Active => {
                self.filter.status = Some(TaskStatus::Pending);
                self.filter.is_active = Some(true);
            }
            StatusFilterType::Overdue => {
                self.filter.status = Some(TaskStatus::Pending);
                self.filter.is_overdue = Some(true);
            }
            StatusFilterType::Waiting => {
                self.filter.status = Some(TaskStatus::Waiting);
            }
            StatusFilterType::Deleted => {
                self.filter.status = Some(TaskStatus::Deleted);
            }
        }
    }

    fn apply_current_field(&mut self) {
        match self.active_field {
            FilterField::Project => {
                self.filter.project = if self.project_input.trim().is_empty() {
                    None
                } else {
                    Some(self.project_input.trim().to_string())
                };
            }
            FilterField::Tags => {
                self.filter.tags = self.tags_input
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            }
            FilterField::Description => {
                self.filter.description_contains = if self.description_input.trim().is_empty() {
                    None
                } else {
                    Some(self.description_input.trim().to_string())
                };
            }
            _ => {} // Priority and Status are handled in real-time
        }
    }

    pub fn get_filter(&self) -> &TaskFilter {
        &self.filter
    }

    pub fn clear_filters(&mut self) {
        self.filter = TaskFilter::default();
        self.project_input.clear();
        self.tags_input.clear();
        self.description_input.clear();
        self.status_filter_type = StatusFilterType::Pending; // Reset to default
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if !self.is_visible {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Status
                Constraint::Percentage(15), // Priority
                Constraint::Percentage(25), // Project
                Constraint::Percentage(20), // Tags
                Constraint::Percentage(20), // Description
            ])
            .split(area);

        // Status field
        self.render_status_field(f, chunks[0]);
        
        // Priority field
        self.render_priority_field(f, chunks[1]);
        
        // Project field with hints
        self.render_project_field(f, chunks[2]);
        
        // Tags field with hints
        self.render_tags_field(f, chunks[3]);
        
        // Description field
        self.render_text_field(f, chunks[4], "Description", &self.description_input, FilterField::Description);
    }

    fn render_status_field(&self, f: &mut Frame, area: Rect) {
        let status_text = match self.status_filter_type {
            StatusFilterType::All => "All",
            StatusFilterType::Pending => "Pending",
            StatusFilterType::Completed => "Completed",
            StatusFilterType::Active => "Active",
            StatusFilterType::Overdue => "Overdue",
            StatusFilterType::Waiting => "Waiting",
            StatusFilterType::Deleted => "Deleted",
        };

        let is_active = matches!(self.active_field, FilterField::Status);
        let style = if is_active && self.is_editing {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else if is_active {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(format!("Status: {}", status_text))
            .block(Block::default().borders(Borders::ALL))
            .style(style);

        f.render_widget(paragraph, area);
    }

    fn render_priority_field(&self, f: &mut Frame, area: Rect) {
        let priority_text = match self.filter.priority {
            Some(Priority::High) => "High",
            Some(Priority::Medium) => "Medium",
            Some(Priority::Low) => "Low",
            None => "Any",
        };

        let is_active = matches!(self.active_field, FilterField::Priority);
        let style = if is_active && self.is_editing {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else if is_active {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(format!("Priority: {}", priority_text))
            .block(Block::default().borders(Borders::ALL))
            .style(style);

        f.render_widget(paragraph, area);
    }

    fn render_project_field(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_field == FilterField::Project;
        let style = if is_active && self.is_editing {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else if is_active {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };

        let title = if is_active && self.is_editing {
            "Project (editing)".to_string()
        } else {
            "Project".to_string()
        };

        // Show current input and available projects as hint
        let mut text_lines = vec![self.project_input.clone()];
        
        if is_active && !self.available_projects.is_empty() {
            text_lines.push("".to_string()); // Empty line
            text_lines.push("Available:".to_string());
            
            // Show first few available projects as hints
            let max_show = 3;
            for project in self.available_projects.iter().take(max_show) {
                text_lines.push(format!("• {}", project));
            }
            
            if self.available_projects.len() > max_show {
                text_lines.push(format!("... and {} more", self.available_projects.len() - max_show));
            }
        }

        let text = text_lines.join("\n");
        let paragraph = Paragraph::new(text)
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(style);
        f.render_widget(paragraph, area);
    }

    fn render_tags_field(&self, f: &mut Frame, area: Rect) {
        let is_active = self.active_field == FilterField::Tags;
        let style = if is_active && self.is_editing {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else if is_active {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };

        let title = if is_active && self.is_editing {
            "Tags (editing)".to_string()
        } else {
            "Tags".to_string()
        };

        // Show current input and available tags as hint
        let mut text_lines = vec![self.tags_input.clone()];
        
        if is_active && !self.available_tags.is_empty() {
            text_lines.push("".to_string()); // Empty line
            text_lines.push("Available:".to_string());
            
            // Show first few available tags as hints
            let max_show = 3;
            for tag in self.available_tags.iter().take(max_show) {
                text_lines.push(format!("• {}", tag));
            }
            
            if self.available_tags.len() > max_show {
                text_lines.push(format!("... and {} more", self.available_tags.len() - max_show));
            }
        }

        let text = text_lines.join("\n");
        let paragraph = Paragraph::new(text)
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(style);
        f.render_widget(paragraph, area);
    }

    fn render_text_field(&self, f: &mut Frame, area: Rect, label: &str, value: &str, field: FilterField) {
        let is_active = self.active_field == field;
        let style = if is_active && self.is_editing {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else if is_active {
            Style::default().bg(Color::DarkGray).fg(Color::White)  
        } else {
            Style::default()
        };

        let display_value = if value.is_empty() { "Any" } else { value };
        let paragraph = Paragraph::new(format!("{}: {}", label, display_value))
            .block(Block::default().borders(Borders::ALL))
            .style(style);

        f.render_widget(paragraph, area);
    }
}
