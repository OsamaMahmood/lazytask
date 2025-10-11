// Task form dialog for adding/editing tasks

use anyhow::Result;
use chrono::{NaiveDate, TimeZone, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::data::models::{Priority, Task};
use crate::handlers::input::Action;

#[derive(Debug, Clone)]
pub enum FormField {
    Description,
    Project,
    Priority,
    Due,
    Tags,
}

pub struct TaskForm {
    pub task: Task,
    pub active_field: FormField,
    pub is_editing: bool,
    pub description_input: String,
    pub project_input: String,
    pub tags_input: String,
    pub due_input: String,
    pub priority_index: usize,
}

impl TaskForm {
    pub fn new_task() -> Self {
        TaskForm {
            task: Task::new("".to_string()),
            active_field: FormField::Description,
            is_editing: true, // Start editing immediately
            description_input: String::new(),
            project_input: String::new(),
            tags_input: String::new(),
            due_input: String::new(),
            priority_index: 0, // None, H, M, L
        }
    }

    pub fn edit_task(task: Task) -> Self {
        let priority_index = match task.priority {
            None => 0,
            Some(Priority::High) => 1,
            Some(Priority::Medium) => 2,
            Some(Priority::Low) => 3,
        };

        let tags_str = task.tags.join(" ");
        let due_str = task.due
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        TaskForm {
            description_input: task.description.clone(),
            project_input: task.project.clone().unwrap_or_default(),
            tags_input: tags_str,
            due_input: due_str,
            task,
            active_field: FormField::Description,
            is_editing: true, // Start editing immediately
            priority_index,
        }
    }

    pub fn handle_input(&mut self, action: Action) -> Result<Option<TaskFormResult>> {
        match action {
            Action::Back => {
                return Ok(Some(TaskFormResult::Cancel));
            }
            Action::Select => {
                if self.is_editing {
                    self.is_editing = false;
                } else {
                    // Validate before saving
                    if self.description_input.trim().is_empty() {
                        // Don't save if description is empty, maybe show error?
                        // For now, switch to editing description field
                        self.active_field = FormField::Description;
                        self.is_editing = true;
                    } else {
                        return Ok(Some(TaskFormResult::Save(self.build_task())));
                    }
                }
            }
            Action::MoveDown => {
                self.next_field();
                self.is_editing = true; // Auto-enter editing mode
            }
            Action::MoveUp => {
                self.previous_field();
                self.is_editing = true; // Auto-enter editing mode
            }
            Action::MoveRight => {
                // Just ensure editing mode is on
                self.is_editing = true;
            }
            Action::Character(c) => {
                // Auto-enter editing mode if not already editing
                self.is_editing = true;
                match self.active_field {
                    FormField::Description => self.description_input.push(c),
                    FormField::Project => self.project_input.push(c),
                    FormField::Tags => self.tags_input.push(c),
                    FormField::Due => self.due_input.push(c),
                    FormField::Priority => {
                        // Priority field uses index, handle separately
                        match c.to_ascii_uppercase() {
                            'H' => self.priority_index = 1,
                            'M' => self.priority_index = 2,
                            'L' => self.priority_index = 3,
                            'N' => self.priority_index = 0,
                            _ => {}
                        }
                    }
                }
            }
            Action::Backspace => {
                // Auto-enter editing mode if not already editing
                self.is_editing = true;
                match self.active_field {
                    FormField::Description => { self.description_input.pop(); }
                    FormField::Project => { self.project_input.pop(); }
                    FormField::Tags => { self.tags_input.pop(); }
                    FormField::Due => { self.due_input.pop(); }
                    FormField::Priority => {
                        // Reset priority to None
                        self.priority_index = 0;
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn next_field(&mut self) {
        self.active_field = match self.active_field {
            FormField::Description => FormField::Project,
            FormField::Project => FormField::Priority,
            FormField::Priority => FormField::Due,
            FormField::Due => FormField::Tags,
            FormField::Tags => FormField::Description,
        };
    }

    fn previous_field(&mut self) {
        self.active_field = match self.active_field {
            FormField::Description => FormField::Tags,
            FormField::Project => FormField::Description,
            FormField::Priority => FormField::Project,
            FormField::Due => FormField::Priority,
            FormField::Tags => FormField::Due,
        };
    }

    fn build_task(&self) -> Task {
        let mut task = self.task.clone();
        task.description = self.description_input.clone();
        task.project = if self.project_input.is_empty() {
            None
        } else {
            Some(self.project_input.clone())
        };
        
        task.priority = match self.priority_index {
            1 => Some(Priority::High),
            2 => Some(Priority::Medium), 
            3 => Some(Priority::Low),
            _ => None,
        };

        task.tags = if self.tags_input.is_empty() {
            Vec::new()
        } else {
            self.tags_input.split_whitespace().map(|s| s.to_string()).collect()
        };

        // Parse due date from due_input string
        if !self.due_input.trim().is_empty() {
            // Try to parse various date formats
            if let Ok(parsed_date) = NaiveDate::parse_from_str(&self.due_input, "%Y-%m-%d") {
                task.due = Some(Utc.from_utc_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap()));
            } else if let Ok(parsed_date) = NaiveDate::parse_from_str(&self.due_input, "%m/%d/%Y") {
                task.due = Some(Utc.from_utc_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap()));
            } else if let Ok(parsed_date) = NaiveDate::parse_from_str(&self.due_input, "%d-%m-%Y") {
                task.due = Some(Utc.from_utc_datetime(&parsed_date.and_hms_opt(0, 0, 0).unwrap()));
            }
            // If parsing fails, due remains None (could add error handling here)
        }

        task
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create centered dialog
        let popup_area = Self::centered_rect(60, 70, area);
        
        // Clear the background
        f.render_widget(Clear, popup_area);
        
        // Main container
        let block = Block::default()
            .title("Task Details")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        f.render_widget(block, popup_area);

        // Split into form fields
        let inner_area = popup_area.inner(&Margin {
            vertical: 1,
            horizontal: 2,
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Description
                Constraint::Length(3), // Project
                Constraint::Length(3), // Priority
                Constraint::Length(3), // Due
                Constraint::Length(3), // Tags
                Constraint::Min(2),    // Instructions
            ])
            .split(inner_area);

        // Description field
        self.render_field(
            f,
            chunks[0],
            "Description:",
            &self.description_input,
            matches!(self.active_field, FormField::Description),
        );

        // Project field
        self.render_field(
            f,
            chunks[1],
            "Project:",
            &self.project_input,
            matches!(self.active_field, FormField::Project),
        );

        // Priority field
        let priority_text = match self.priority_index {
            1 => "High",
            2 => "Medium",
            3 => "Low",
            _ => "None",
        };
        self.render_field(
            f,
            chunks[2],
            "Priority:",
            priority_text,
            matches!(self.active_field, FormField::Priority),
        );

        // Due field
        self.render_field(
            f,
            chunks[3],
            "Due:",
            &self.due_input,
            matches!(self.active_field, FormField::Due),
        );

        // Tags field
        self.render_field(
            f,
            chunks[4],
            "Tags:",
            &self.tags_input,
            matches!(self.active_field, FormField::Tags),
        );

        // Instructions
        let instructions = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" Navigate fields  "),
                Span::styled("Type", Style::default().fg(Color::Yellow)),
                Span::raw(" to edit  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" Save  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(" Cancel"),
            ]),
        ])
        .alignment(Alignment::Center);
        f.render_widget(instructions, chunks[5]);
    }

    fn render_field(&self, f: &mut Frame, area: Rect, label: &str, value: &str, is_active: bool) {
        let (style, border_color) = if is_active && self.is_editing {
            (
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD),
                Color::Green
            )
        } else if is_active {
            (
                Style::default().bg(Color::DarkGray).fg(Color::White),
                Color::Yellow
            )
        } else {
            (Style::default(), Color::Gray)
        };

        let content = format!("{} {}", label, value);
        let paragraph = Paragraph::new(content)
            .style(style)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)))
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(paragraph, area);

        if is_active && self.is_editing {
            let cursor_area = Rect {
                x: area.x + label.len() as u16 + 1 + value.len() as u16 + 1, // +1 for border
                y: area.y + 1, // +1 for border
                width: 1,
                height: 1,
            };
            f.render_widget(
                Paragraph::new("█").style(Style::default().fg(Color::Yellow)),
                cursor_area,
            );
        }
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

#[derive(Debug)]
pub enum TaskFormResult {
    Save(Task),
    Cancel,
}
