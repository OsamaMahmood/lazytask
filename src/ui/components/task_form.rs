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
    // Cursor positions for each text field
    pub description_cursor: usize,
    pub project_cursor: usize,
    pub tags_cursor: usize,
    pub due_cursor: usize,
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
            // Initialize cursors at end of text
            description_cursor: 0,
            project_cursor: 0,
            tags_cursor: 0,
            due_cursor: 0,
        }
    }

    pub fn edit_task(task: Task) -> Self {
        let priority_index = match task.priority {
            None => 0,
            Some(Priority::High) => 1,
            Some(Priority::Medium) => 2,
            Some(Priority::Low) => 3,
        };

        let tags_str = task.tags.join(", ");
        let due_str = task.due
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        let description_text = task.description.clone();
        let project_text = task.project.clone().unwrap_or_default();
        
        TaskForm {
            description_input: description_text.clone(),
            project_input: project_text.clone(),
            tags_input: tags_str.clone(),
            due_input: due_str.clone(),
            task,
            active_field: FormField::Description,
            is_editing: true, // Start editing immediately
            priority_index,
            // Initialize cursors at end of existing text
            description_cursor: description_text.len(),
            project_cursor: project_text.len(),
            tags_cursor: tags_str.len(),
            due_cursor: due_str.len(),
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
            Action::MoveDown | Action::Tab => {
                self.next_field();
                self.is_editing = true; // Auto-enter editing mode
            }
            Action::MoveUp => {
                self.previous_field();
                self.is_editing = true; // Auto-enter editing mode
            }
            Action::Character(c) => {
                // Auto-enter editing mode if not already editing
                self.is_editing = true;
                match self.active_field {
                    FormField::Description => {
                        self.description_input.insert(self.description_cursor, c);
                        self.description_cursor += 1;
                    }
                    FormField::Project => {
                        self.project_input.insert(self.project_cursor, c);
                        self.project_cursor += 1;
                    }
                    FormField::Tags => {
                        self.tags_input.insert(self.tags_cursor, c);
                        self.tags_cursor += 1;
                    }
                    FormField::Due => {
                        self.due_input.insert(self.due_cursor, c);
                        self.due_cursor += 1;
                    }
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
                    FormField::Description => {
                        if self.description_cursor > 0 {
                            self.description_cursor -= 1;
                            self.description_input.remove(self.description_cursor);
                        }
                    }
                    FormField::Project => {
                        if self.project_cursor > 0 {
                            self.project_cursor -= 1;
                            self.project_input.remove(self.project_cursor);
                        }
                    }
                    FormField::Tags => {
                        if self.tags_cursor > 0 {
                            self.tags_cursor -= 1;
                            self.tags_input.remove(self.tags_cursor);
                        }
                    }
                    FormField::Due => {
                        if self.due_cursor > 0 {
                            self.due_cursor -= 1;
                            self.due_input.remove(self.due_cursor);
                        }
                    }
                    FormField::Priority => {
                        // Reset priority to None
                        self.priority_index = 0;
                    }
                }
            }
            Action::MoveLeft => {
                if self.is_editing {
                    match self.active_field {
                        FormField::Description => {
                            if self.description_cursor > 0 {
                                self.description_cursor -= 1;
                            }
                        }
                        FormField::Project => {
                            if self.project_cursor > 0 {
                                self.project_cursor -= 1;
                            }
                        }
                        FormField::Tags => {
                            if self.tags_cursor > 0 {
                                self.tags_cursor -= 1;
                            }
                        }
                        FormField::Due => {
                            if self.due_cursor > 0 {
                                self.due_cursor -= 1;
                            }
                        }
                        FormField::Priority => {
                            // Priority doesn't use cursor
                        }
                    }
                }
            }
            Action::MoveRight => {
                if self.is_editing {
                    match self.active_field {
                        FormField::Description => {
                            if self.description_cursor < self.description_input.len() {
                                self.description_cursor += 1;
                            }
                        }
                        FormField::Project => {
                            if self.project_cursor < self.project_input.len() {
                                self.project_cursor += 1;
                            }
                        }
                        FormField::Tags => {
                            if self.tags_cursor < self.tags_input.len() {
                                self.tags_cursor += 1;
                            }
                        }
                        FormField::Due => {
                            if self.due_cursor < self.due_input.len() {
                                self.due_cursor += 1;
                            }
                        }
                        FormField::Priority => {
                            // Priority doesn't use cursor
                        }
                    }
                } else {
                    // If not editing, enter editing mode
                    self.is_editing = true;
                }
            }
            Action::Space => {
                // Handle space as a character in forms
                if self.is_editing {
                    match self.active_field {
                        FormField::Description => {
                            self.description_input.insert(self.description_cursor, ' ');
                            self.description_cursor += 1;
                        }
                        FormField::Project => {
                            self.project_input.insert(self.project_cursor, ' ');
                            self.project_cursor += 1;
                        }
                        FormField::Tags => {
                            self.tags_input.insert(self.tags_cursor, ' ');
                            self.tags_cursor += 1;
                        }
                        FormField::Due => {
                            self.due_input.insert(self.due_cursor, ' ');
                            self.due_cursor += 1;
                        }
                        FormField::Priority => {
                            // Priority doesn't use text input
                        }
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
        // Set cursor to end of text for the new field
        self.set_cursor_to_end();
    }

    fn previous_field(&mut self) {
        self.active_field = match self.active_field {
            FormField::Description => FormField::Tags,
            FormField::Project => FormField::Description,
            FormField::Priority => FormField::Project,
            FormField::Due => FormField::Priority,
            FormField::Tags => FormField::Due,
        };
        // Set cursor to end of text for the new field
        self.set_cursor_to_end();
    }
    
    fn set_cursor_to_end(&mut self) {
        match self.active_field {
            FormField::Description => {
                self.description_cursor = self.description_input.len();
            }
            FormField::Project => {
                self.project_cursor = self.project_input.len();
            }
            FormField::Tags => {
                self.tags_cursor = self.tags_input.len();
            }
            FormField::Due => {
                self.due_cursor = self.due_input.len();
            }
            FormField::Priority => {
                // Priority doesn't use cursor
            }
        }
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

        task.tags = if self.tags_input.trim().is_empty() {
            Vec::new()
        } else {
            // Handle both space-separated and comma-separated tags
            // Split on both whitespace and commas, then filter out empty strings
            self.tags_input
                .split(|c: char| c == ',' || c.is_whitespace())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
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
        // Responsive dialog sizing based on terminal size
        let (width_pct, height_pct) = if area.width < 80 {
            (90, 80)  // Nearly full screen on very narrow terminals
        } else if area.width < 120 {
            (80, 75)  // Large dialog on narrow terminals
        } else if area.width < 180 {
            (70, 70)  // Medium dialog on medium terminals
        } else {
            (60, 65)  // Standard dialog on wide terminals
        };
        
        let popup_area = Self::centered_rect(width_pct, height_pct, area);
        
        // Clear the background
        f.render_widget(Clear, popup_area);
        
        // Main container with better visibility
        let block = Block::default()
            .title("Task Details")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));
        f.render_widget(block, popup_area);

        // Split into form fields
        let inner_area = popup_area.inner(&Margin {
            vertical: 1,
            horizontal: 2,
        });

        // Responsive field sizing based on available space
        let field_height = if inner_area.height < 15 {
            2  // Compact fields for very small dialogs
        } else {
            3  // Standard field height
        };

        let instruction_space = if inner_area.height < 20 {
            Constraint::Min(1)     // Minimal instruction area
        } else {
            Constraint::Min(3)     // Standard instruction area
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(field_height), // Description
                Constraint::Length(field_height), // Project
                Constraint::Length(field_height), // Priority
                Constraint::Length(field_height), // Due
                Constraint::Length(field_height), // Tags
                instruction_space,                 // Instructions (responsive)
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

        // Instructions with enhanced cursor movement capabilities
        let instructions = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" Navigate fields  ", Style::default().fg(Color::White)),
                Span::styled("←→", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled(" Move cursor  ", Style::default().fg(Color::White)),
                Span::styled("Type", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" to edit  ", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" Save  ", Style::default().fg(Color::White)),
                Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(" Cancel  ", Style::default().fg(Color::White)),
                Span::styled("Backspace", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" Delete", Style::default().fg(Color::White)),
            ]),
        ])
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Center);
        f.render_widget(instructions, chunks[5]);
    }

    fn render_field(&self, f: &mut Frame, area: Rect, label: &str, value: &str, is_active: bool) {
        let (style, border_color) = if is_active && self.is_editing {
            (
                Style::default().bg(Color::Black).fg(Color::Green).add_modifier(Modifier::BOLD),
                Color::Green
            )
        } else if is_active {
            (
                Style::default().bg(Color::Black).fg(Color::Yellow).add_modifier(Modifier::BOLD),
                Color::Yellow
            )
        } else {
            (Style::default().bg(Color::Black).fg(Color::White), Color::Gray)
        };

        let content = format!("{} {}", label, value);
        let paragraph = Paragraph::new(content)
            .style(style)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(border_color)))
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(paragraph, area);

        if is_active && self.is_editing {
            let cursor_pos = self.get_cursor_position_for_field();
            let cursor_area = Rect {
                x: area.x + label.len() as u16 + 1 + cursor_pos as u16 + 1, // Position cursor at cursor_pos
                y: area.y + 1, // +1 for border
                width: 1,
                height: 1,
            };
            f.render_widget(
                Paragraph::new("█").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                cursor_area,
            );
        }
    }
    
    fn get_cursor_position_for_field(&self) -> usize {
        match self.active_field {
            FormField::Description => self.description_cursor,
            FormField::Project => self.project_cursor,
            FormField::Tags => self.tags_cursor,
            FormField::Due => self.due_cursor,
            FormField::Priority => 0, // Priority doesn't use cursor
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
