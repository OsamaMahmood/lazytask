// Comprehensive task detail view component

use chrono::Utc;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::data::models::{Task, TaskStatus, Priority};

pub struct TaskDetailWidget;

impl TaskDetailWidget {
    pub fn new() -> Self {
        TaskDetailWidget
    }

    pub fn render(&self, f: &mut Frame, area: Rect, task: Option<&Task>) {
        if let Some(task) = task {
            self.render_task_details(f, area, task);
        } else {
            let placeholder = Paragraph::new("Select a task to view details")
                .block(Block::default().title("Task Details").borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(placeholder, area);
        }
    }

    fn render_task_details(&self, f: &mut Frame, area: Rect, task: &Task) {
        // Split the area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(20),  // Main details section
                Constraint::Min(5),   // Modification history
            ])
            .split(area);

        // Render main details
        self.render_main_details(f, chunks[0], task);
        
        // Render modification history
        self.render_modification_history(f, chunks[1], task);
    }

    fn render_main_details(&self, f: &mut Frame, area: Rect, task: &Task) {
        let mut lines = Vec::new();
        
        // Header
        lines.push(Line::from(vec![
            Span::styled("Name", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("          "),
            Span::styled("Value", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        
        // ID
        lines.push(Line::from(vec![
            Span::styled("ID            ", Style::default().fg(Color::Cyan)),
            Span::styled(task.id.map(|i| i.to_string()).unwrap_or_else(|| "".to_string()), Style::default().fg(Color::White)),
        ]));
        
        // Description
        lines.push(Line::from(vec![
            Span::styled("Description   ", Style::default().fg(Color::Cyan)),
            Span::styled(&task.description, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));
        
        // Status
        let (status_str, status_color) = match task.status {
            TaskStatus::Pending => ("Pending", Color::Yellow),
            TaskStatus::Completed => ("Completed", Color::Green),
            TaskStatus::Deleted => ("Deleted", Color::Red),
            TaskStatus::Waiting => ("Waiting", Color::Magenta),
            TaskStatus::Recurring => ("Recurring", Color::Blue),
        };
        lines.push(Line::from(vec![
            Span::styled("Status        ", Style::default().fg(Color::Cyan)),
            Span::styled(status_str, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]));
        
        // Project
        if let Some(ref project) = task.project {
            lines.push(Line::from(vec![
                Span::styled("Project       ", Style::default().fg(Color::Cyan)),
                Span::styled(project, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]));
        }
        
        // Priority
        if let Some(ref priority) = task.priority {
            let (priority_str, priority_color) = match priority {
                Priority::High => ("High", Color::Red),
                Priority::Medium => ("Medium", Color::Yellow),
                Priority::Low => ("Low", Color::Green),
            };
            lines.push(Line::from(vec![
                Span::styled("Priority      ", Style::default().fg(Color::Cyan)),
                Span::styled(priority_str, Style::default().fg(priority_color).add_modifier(Modifier::BOLD)),
            ]));
        }
        
        // Due date
        if let Some(due) = task.due {
            let due_color = if task.is_overdue() {
                Color::Red
            } else {
                Color::Yellow
            };
            lines.push(Line::from(vec![
                Span::styled("Due           ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    due.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(due_color).add_modifier(Modifier::BOLD)
                ),
            ]));
        }
        
        // Get current time for relative calculations
        let now = Utc::now();
        
        // Start date (when task is started)
        if let Some(start) = task.start {
            let start_duration = now - start;
            let start_relative = self.format_relative_time(start_duration);
            lines.push(Line::from(vec![
                Span::styled("Start         ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("{} ({})", 
                    start.format("%Y-%m-%d %H:%M:%S"), 
                    start_relative
                ), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]));
        }
        
        // Created (formerly Entered)
        let entry_duration = now - task.entry;
        let entry_relative = self.format_relative_time(entry_duration);
        lines.push(Line::from(vec![
            Span::styled("Created       ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{} ({})", 
                task.entry.format("%Y-%m-%d %H:%M:%S"), 
                entry_relative
            ), Style::default().fg(Color::Gray)),
        ]));
        
        // Last modified
        if let Some(modified) = task.modified {
            let mod_duration = now - modified;
            let mod_relative = self.format_relative_time(mod_duration);
            lines.push(Line::from(vec![
                Span::styled("Last modified ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("{} ({})", 
                    modified.format("%Y-%m-%d %H:%M:%S"), 
                    mod_relative
                ), Style::default().fg(Color::Gray)),
            ]));
        }
        
        // Tags
        if !task.tags.is_empty() {
            let tags_str = task.tags.join(" ");
            lines.push(Line::from(vec![
                Span::styled("Tags          ", Style::default().fg(Color::Cyan)),
                Span::styled(tags_str, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]));
        }
        
        // UUID
        lines.push(Line::from(vec![
            Span::styled("UUID          ", Style::default().fg(Color::Cyan)),
            Span::styled(&task.uuid, Style::default().fg(Color::DarkGray)),
        ]));
        
        // Urgency
        let urgency_color = if task.urgency >= 10.0 {
            Color::Red
        } else if task.urgency >= 5.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        lines.push(Line::from(vec![
            Span::styled("Urgency       ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:.1}", task.urgency), Style::default().fg(urgency_color).add_modifier(Modifier::BOLD)),
        ]));

        let detail = Paragraph::new(lines)
            .block(Block::default().title("Task Details").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(detail, area);
    }

    fn render_modification_history(&self, f: &mut Frame, area: Rect, task: &Task) {
        let mut header = Vec::new();
        header.push(Line::from(vec![
            Span::styled("Date", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("                Modification", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        
        // Collect modifications with latest first
        let mut modifications = Vec::new();
        
        // Use modified date if available, otherwise use entry date
        let display_date = if let Some(modified) = task.modified {
            modified.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            task.entry.format("%Y-%m-%d %H:%M:%S").to_string()
        };
        
        // Show latest modifications first (most recent changes)
        
        // Due date changes (show with modified date if changed, or entry date if set on creation)
        if let Some(due) = task.due {
            let due_display_date = if let Some(modified) = task.modified {
                modified.format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                task.entry.format("%Y-%m-%d %H:%M:%S").to_string()
            };
            modifications.push(Line::from(vec![
                Span::styled(due_display_date, Style::default().fg(Color::Gray)),
                Span::styled(" Due set to '", Style::default().fg(Color::Gray)),
                Span::styled(due.format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("'.", Style::default().fg(Color::Gray)),
            ]));
        }
        
        // Start date (when task is started - IMPORTANT!)
        if let Some(start) = task.start {
            modifications.push(Line::from(vec![
                Span::styled(start.format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().fg(Color::Gray)),
                Span::styled(" Start set to '", Style::default().fg(Color::Gray)),
                Span::styled(start.format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("'.", Style::default().fg(Color::Gray)),
            ]));
        }
        
        // Tags (typically added during modifications)
        for tag in &task.tags {
            modifications.push(Line::from(vec![
                Span::styled(display_date.clone(), Style::default().fg(Color::Gray)),
                Span::styled(" Tag '", Style::default().fg(Color::Gray)),
                Span::styled(tag, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled("' added.", Style::default().fg(Color::Gray)),
            ]));
        }
        
        // Priority
        if let Some(ref priority) = task.priority {
            modifications.push(Line::from(vec![
                Span::styled(display_date.clone(), Style::default().fg(Color::Gray)),
                Span::styled(" Priority set to '", Style::default().fg(Color::Gray)),
                Span::styled(match priority {
                    Priority::High => "High",
                    Priority::Medium => "Medium",
                    Priority::Low => "Low",
                }, Style::default().fg(match priority {
                    Priority::High => Color::Red,
                    Priority::Medium => Color::Yellow,
                    Priority::Low => Color::Green,
                }).add_modifier(Modifier::BOLD)),
                Span::styled("'.", Style::default().fg(Color::Gray)),
            ]));
        }
        
        // Status
        modifications.push(Line::from(vec![
            Span::styled(display_date.clone(), Style::default().fg(Color::Gray)),
            Span::styled(" Status set to '", Style::default().fg(Color::Gray)),
            Span::styled(match task.status {
                TaskStatus::Pending => "pending",
                TaskStatus::Completed => "completed",
                TaskStatus::Deleted => "deleted",
                TaskStatus::Waiting => "waiting",
                TaskStatus::Recurring => "recurring",
            }, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("'.", Style::default().fg(Color::Gray)),
        ]));
        
        // Project
        if let Some(ref project) = task.project {
            modifications.push(Line::from(vec![
                Span::styled(display_date.clone(), Style::default().fg(Color::Gray)),
                Span::styled(" Project set to '", Style::default().fg(Color::Gray)),
                Span::styled(project, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("'.", Style::default().fg(Color::Gray)),
            ]));
        }
        
        // Description and entry (oldest - shown last)
        modifications.push(Line::from(vec![
            Span::styled(task.entry.format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().fg(Color::Gray)),
            Span::styled(" Description set to '", Style::default().fg(Color::Gray)),
            Span::styled(&task.description, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("'.", Style::default().fg(Color::Gray)),
        ]));
        modifications.push(Line::from(vec![
            Span::styled("                    Entry set to '", Style::default().fg(Color::Gray)),
            Span::styled(task.entry.format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().fg(Color::White)),
            Span::styled("'.", Style::default().fg(Color::Gray)),
        ]));
        
        // Combine header and modifications
        let mut lines = header;
        lines.extend(modifications);

        let history_block = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(history_block, area);
    }

    fn format_relative_time(&self, duration: chrono::Duration) -> String {
        if duration.num_minutes() < 60 {
            format!("{}min", duration.num_minutes().max(1))
        } else if duration.num_hours() < 24 {
            format!("{}h", duration.num_hours())
        } else if duration.num_days() < 30 {
            format!("{}d", duration.num_days())
        } else if duration.num_days() < 365 {
            let weeks = duration.num_days() / 7;
            if weeks < 10 {
                format!("{}w", weeks)
            } else {
                format!("{}mo", duration.num_days() / 30)
            }
        } else {
            format!("{}y", duration.num_days() / 365)
        }
    }
}
