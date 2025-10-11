// Task display widget

use chrono::Utc;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::data::models::Task;

pub struct TaskListWidget {
    pub state: TableState,
    tasks: Vec<Task>,
}

impl TaskListWidget {
    pub fn new() -> Self {
        TaskListWidget {
            state: TableState::default(),
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
        let header_cells = ["  ID", " Age    ", "Status", "P", " Project       ", " Tag        ", "   Due   ", " Description          ", " Urg"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .collect::<Vec<_>>();

        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1)
            .bottom_margin(1);

        let rows: Vec<Row> = self
            .tasks
            .iter()
            .map(|task| {
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
                
                let tags_raw = if task.tags.is_empty() {
                    "".to_string()
                } else {
                    task.tags.join(",")
                };
                
                let due = if let Some(due) = task.due {
                    // Show compact due date format
                    let now = Utc::now();
                    let days_until_due = (due.date_naive() - now.date_naive()).num_days();
                    
                    if days_until_due < 0 {
                        format!("{}d", days_until_due) // Overdue (negative)
                    } else if days_until_due <= 7 {
                        format!("{}d", days_until_due) // Within a week
                    } else {
                        due.format("%m/%d").to_string() // Further out, show date
                    }
                } else {
                    "".to_string()
                };

                let urgency = format!("{:.1}", task.urgency);

                // Truncate description if too long  
                let description = if task.description.len() > 24 {
                    format!("{}...", &task.description[..21])
                } else {
                    task.description.clone()
                };

                // Truncate project name if too long
                let project = if project_raw.len() > 14 {
                    format!("{}...", &project_raw[..11])
                } else {
                    project_raw.to_string()
                };

                // Truncate tags if too long
                let tags = if tags_raw.len() > 11 {
                    format!("{}...", &tags_raw[..8])
                } else {
                    tags_raw
                };

                Row::new(vec![
                    Cell::from(format!("{:>3} ", id)),        // Right-aligned ID with padding
                    Cell::from(format!(" {:<6}", age)),       // Left-aligned age with padding
                    Cell::from(format!(" {:<5} ", status)),   // Left-aligned status with padding
                    Cell::from(format!("{:^1}", priority)),   // Center-aligned priority
                    Cell::from(format!(" {:<14}", project)),  // Left-aligned project with padding
                    Cell::from(format!(" {:<11}", tags)),     // Left-aligned tags with padding
                    Cell::from(format!("{:^8}", due)),        // Center-aligned due
                    Cell::from(format!(" {:<23}", description)), // Left-aligned description with padding
                    Cell::from(format!("{:>4}", urgency)),    // Right-aligned urgency
                ])
                .height(1)
            })
            .collect();

        let table = Table::new(rows)
            .header(header)
            .block(Block::default()
                .title(" Tasks ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
            )
            .widths(&[
                Constraint::Length(5),   // ID
                Constraint::Length(8),   // Age  
                Constraint::Length(8),   // Status
                Constraint::Length(4),   // Priority
                Constraint::Length(16),  // Project
                Constraint::Length(13),  // Tags
                Constraint::Length(10),  // Due
                Constraint::Min(25),     // Description (flexible)
                Constraint::Length(6),   // Urgency
            ])
            .column_spacing(1)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(table, area, &mut self.state);
    }
}
