// Task display widget with clean, template-like table configuration

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
        let formatter = TaskTableFormatter::new();
        
        // Create clean, minimal headers
        let header_cells = formatter.headers()
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .collect::<Vec<_>>();

        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1);

        // Create data rows using the formatter
        let rows: Vec<Row> = self.tasks
            .iter()
            .map(|task| formatter.format_task_row(task))
            .collect();

        let column_widths = formatter.column_widths();
        let table = Table::new(rows)
            .header(header)
            .block(Block::default()
                .title(" Tasks ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
            )
            .widths(&column_widths)
            .column_spacing(2)  // Clean spacing between columns
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(table, area, &mut self.state);
    }
}

// Clean, template-like table configuration - much better than hardcoding!
struct TaskTableFormatter;

impl TaskTableFormatter {
    fn new() -> Self {
        TaskTableFormatter
    }
    
    // Define column headers - easy to modify
    fn headers(&self) -> [&'static str; 9] {
        ["ID", "Age", "Status", "P", "Project", "Tag", "Due", "Description", "Urg"]
    }
    
    // Define column widths - optimized for readability with more space for description
    fn column_widths(&self) -> [Constraint; 9] {
        [
            Constraint::Length(4),   // ID - minimal
            Constraint::Length(4),   // Age - compact  
            Constraint::Length(9),   // Status - minimal (St)
            Constraint::Length(2),   // Priority - minimal
            Constraint::Length(15),  // Project - reduced from 16
            Constraint::Length(8),   // Tags - reduced from 14
            Constraint::Length(6),   // Due - reduced from 12
            Constraint::Min(40),     // Description - MUCH MORE SPACE!
            Constraint::Length(5),   // Urgency - compact
        ]
    }
    
    // Format a complete task row with proper spacing
    fn format_task_row(&self, task: &Task) -> Row {
        let cells = vec![
            Cell::from(self.format_id(task.id)),
            Cell::from(self.format_age(task.entry)), 
            Cell::from(self.format_status(&task.status)),
            Cell::from(self.format_priority(&task.priority)),
            Cell::from(self.format_project(&task.project)),
            Cell::from(self.format_tags(&task.tags)),
            Cell::from(self.format_due(task.due)),
            Cell::from(self.format_description(&task.description)),
            Cell::from(self.format_urgency(task.urgency)),
        ];
        Row::new(cells).height(1)
    }
    
    // Individual field formatters - clean and maintainable
    fn format_id(&self, id: Option<u32>) -> String {
        id.map(|i| i.to_string()).unwrap_or_else(|| "".to_string())
    }
    
    fn format_age(&self, entry: chrono::DateTime<Utc>) -> String {
        let now = Utc::now();
        let duration = now - entry;
        
        if duration.num_minutes() < 60 {
            format!("{}m", duration.num_minutes().max(1))
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
    
    fn format_status(&self, status: &crate::data::models::TaskStatus) -> String {
        match status {
            crate::data::models::TaskStatus::Pending => "Pending".to_string(),
            crate::data::models::TaskStatus::Completed => "Completed".to_string(),
            crate::data::models::TaskStatus::Deleted => "Deleted".to_string(),
            crate::data::models::TaskStatus::Waiting => "Waiting".to_string(),
            crate::data::models::TaskStatus::Recurring => "Recurring".to_string(),
        }
    }
    
    fn format_priority(&self, priority: &Option<crate::data::models::Priority>) -> String {
        priority.as_ref()
            .map(|p| p.as_char().to_string())
            .unwrap_or_else(|| " ".to_string())
    }
    
    fn format_project(&self, project: &Option<String>) -> String {
        project.as_deref()
            .map(|p| if p.len() > 9 { format!("{}...", &p[..6]) } else { p.to_string() })
            .unwrap_or_else(|| "".to_string())
    }
    
    fn format_tags(&self, tags: &[String]) -> String {
        if tags.is_empty() {
            "".to_string()
        } else {
            let joined = tags.join(",");
            if joined.len() > 7 { 
                format!("{}...", &joined[..4])
            } else { 
                joined 
            }
        }
    }
    
    fn format_due(&self, due: Option<chrono::DateTime<Utc>>) -> String {
        if let Some(due) = due {
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
        }
    }
    
    fn format_description(&self, description: &str) -> String {
        // Much more space for description - up to 39 characters!
        if description.len() > 39 {
            format!("{}...", &description[..36])
        } else {
            description.to_string()
        }
    }
    
    fn format_urgency(&self, urgency: f64) -> String {
        format!("{:.1}", urgency)
    }
}