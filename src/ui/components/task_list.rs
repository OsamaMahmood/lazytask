// Task display widget

use chrono::Utc;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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
        // Create header with proper markdown-style table formatting
        let header_line = format!(
            "│{:^6}│{:^8}│{:^8}│{:^3}│{:^16}│{:^14}│{:^12}│{:^30}│{:^6}│",
            "ID", "Age", "Status", "P", "Project", "Tag", "Due", "Description", "Urg"
        );
        
        let separator_line = format!(
            "├{}┼{}┼{}┼{}┼{}┼{}┼{}┼{}┼{}┤",
            "─".repeat(6), "─".repeat(8), "─".repeat(8), "─".repeat(3),
            "─".repeat(16), "─".repeat(14), "─".repeat(12), "─".repeat(30), "─".repeat(6)
        );

        let mut items = Vec::new();
        
        // Add header
        items.push(ListItem::new(Line::from(vec![
            Span::styled(header_line, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        ])));
        
        // Add separator
        items.push(ListItem::new(Line::from(vec![
            Span::styled(separator_line, Style::default().fg(Color::Gray))
        ])));

        // Add task rows
        for task in &self.tasks {
            let id = task.id.map(|id| id.to_string()).unwrap_or_else(|| "".to_string());
            
            // Calculate age (time since entry)
            let age = {
                let now = Utc::now();
                let duration = now - task.entry;
                
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
                        let months = duration.num_days() / 30;
                        format!("{}mth", months)
                    }
                } else {
                    format!("{}y", duration.num_days() / 365)
                }
            };
            
            let status = match task.status {
                crate::data::models::TaskStatus::Pending => "P",
                crate::data::models::TaskStatus::Completed => "C", 
                crate::data::models::TaskStatus::Deleted => "D",
                crate::data::models::TaskStatus::Waiting => "W",
                crate::data::models::TaskStatus::Recurring => "R",
            };
            
            let priority = task.priority.as_ref()
                .map(|p| p.as_char().to_string())
                .unwrap_or_else(|| " ".to_string());
            
            let project_raw = task.project.as_deref().unwrap_or("");
            let project = if project_raw.len() > 15 {
                format!("{}...", &project_raw[..12])
            } else {
                project_raw.to_string()
            };
            
            let tags_raw = if task.tags.is_empty() {
                "".to_string()
            } else {
                task.tags.join(",")
            };
            let tags = if tags_raw.len() > 13 {
                format!("{}...", &tags_raw[..10])
            } else {
                tags_raw
            };
            
            let due = if let Some(due) = task.due {
                let now = Utc::now();
                let days_until_due = (due.date_naive() - now.date_naive()).num_days();
                
                if days_until_due < 0 {
                    format!("{}d", days_until_due)
                } else if days_until_due <= 7 {
                    format!("{}d", days_until_due)
                } else {
                    due.format("%m/%d").to_string()
                }
            } else {
                "".to_string()
            };

            let description = if task.description.len() > 29 {
                format!("{}...", &task.description[..26])
            } else {
                task.description.clone()
            };

            let urgency = format!("{:.1}", task.urgency);
            
            // Format row with proper table styling
            let row_line = format!(
                "│{:^6}│{:<8}│{:^8}│{:^3}│{:<16}│{:<14}│{:^12}│{:<30}│{:>6}│",
                id, age, status, priority, project, tags, due, description, urgency
            );
            
            items.push(ListItem::new(row_line));
        }

        let list = List::new(items)
            .block(Block::default().title(" Tasks ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD))
            .highlight_symbol("» ");

        f.render_stateful_widget(list, area, &mut self.state);
    }
}
