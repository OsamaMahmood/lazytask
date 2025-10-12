// Task display widget with clean, template-like table configuration and intelligent color coding

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

        // Create data rows with intelligent color coding
        let rows: Vec<Row> = self.tasks
            .iter()
            .map(|task| formatter.format_task_row(task))
            .collect();

        let column_widths = formatter.column_widths();
        let task_count = self.tasks.len();
        let title = format!(" Tasks ({}) ", task_count);
        
        let table = Table::new(rows)
            .header(header)
            .block(Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
            )
            .widths(&column_widths)
            .column_spacing(2)  // Clean spacing between columns
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            );

        f.render_stateful_widget(table, area, &mut self.state);
    }
}

// Clean, template-like table configuration with intelligent color coding
struct TaskTableFormatter;

impl TaskTableFormatter {
    fn new() -> Self {
        TaskTableFormatter
    }
    
    // Define column headers - simplified, clean layout
    fn headers(&self) -> [&'static str; 5] {
        ["ID", "Project", "Priority", "Due", "Description"]
    }
    
    // Define column widths - optimized for clean, readable layout
    fn column_widths(&self) -> [Constraint; 5] {
        [
            Constraint::Length(4),   // ID - minimal
            Constraint::Length(15),  // Project - readable
            Constraint::Length(10),  // Priority - full word display
            Constraint::Length(12),  // Due - readable date
            Constraint::Min(40),     // Description - maximum space
        ]
    }
    
    // Format a complete task row with intelligent row-level color coding
    fn format_task_row(&self, task: &Task) -> Row {
        // Determine the most important styling factor for the entire row
        let row_style = self.get_row_style(task);
        
        let cells = vec![
            Cell::from(self.format_id(task.id)),
            Cell::from(self.format_project(&task.project)),
            Cell::from(self.format_priority_full(&task.priority)),
            Cell::from(self.format_due(task.due)),
            Cell::from(self.format_description(&task.description)),
        ];
        Row::new(cells).height(1).style(row_style)
    }
    
    // ===== INTELLIGENT ROW-LEVEL COLOR CODING SYSTEM =====
    
    // Get overall row style based on intelligent task priority hierarchy  
    fn get_row_style(&self, task: &Task) -> Style {
        // Intelligent priority hierarchy combining multiple factors:
        // 1. High priority + overdue/due soon = CRITICAL RED BOLD
        // 2. Any overdue tasks = URGENT RED BOLD  
        // 3. High priority + due within 2 days = URGENT RED BOLD
        // 4. Due today/tomorrow = URGENT YELLOW BOLD
        // 5. High priority tasks = RED
        // 6. Medium priority tasks = YELLOW
        // 7. Completed tasks = DIMMED GRAY
        // 8. Low priority tasks = GREEN
        // 9. Default/no priority tasks = WHITE
        
        let is_high_priority = task.priority == Some(crate::data::models::Priority::High);
        let is_overdue = self.is_overdue(task.due);
        let is_due_today = self.is_due_today(task.due);
        let is_due_within_2_days = self.is_due_within_days(task.due, 2);
        let is_due_tomorrow = self.is_due_tomorrow(task.due);
        
        if is_overdue || is_due_today || (is_high_priority && is_due_within_2_days) {
            // CRITICAL RED: 
            // - All overdue tasks (regardless of priority)
            // - All tasks due today (regardless of priority) 
            // - High priority tasks due within 2 days
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else if is_due_tomorrow {
            // URGENT YELLOW: Due tomorrow = high urgency  
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if is_high_priority {
            // HIGH PRIORITY - Important but not time-critical
            Style::default().fg(Color::Red)
        } else if task.priority == Some(crate::data::models::Priority::Medium) {
            // MEDIUM PRIORITY - Moderate importance
            Style::default().fg(Color::Yellow)
        } else if task.status == crate::data::models::TaskStatus::Completed {
            // COMPLETED - Dimmed
            Style::default().fg(Color::DarkGray)
        } else if task.priority == Some(crate::data::models::Priority::Low) {
            // LOW PRIORITY - Less urgent
            Style::default().fg(Color::Green)
        } else if task.urgency >= 10.0 {
            // HIGH URGENCY (calculated, without explicit priority) 
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            // DEFAULT - Normal tasks
            Style::default().fg(Color::White)
        }
    }
    
    // Helper method to check if task is due soon (today/tomorrow)
    fn is_due_soon(&self, due: Option<chrono::DateTime<Utc>>) -> bool {
        self.is_due_within_days(due, 1) // Today or tomorrow
    }
    
    // Helper method to check if task is due within N days
    fn is_due_within_days(&self, due: Option<chrono::DateTime<Utc>>, days: i64) -> bool {
        if let Some(due_date) = due {
            let now = Utc::now();
            let days_until_due = (due_date.date_naive() - now.date_naive()).num_days();
            days_until_due >= 0 && days_until_due <= days
        } else {
            false
        }
    }
    
    // Helper method to check if task is due today specifically
    fn is_due_today(&self, due: Option<chrono::DateTime<Utc>>) -> bool {
        if let Some(due_date) = due {
            let now = Utc::now();
            let days_until_due = (due_date.date_naive() - now.date_naive()).num_days();
            days_until_due == 0 // Exactly today
        } else {
            false
        }
    }
    
    // Helper method to check if task is due tomorrow specifically
    fn is_due_tomorrow(&self, due: Option<chrono::DateTime<Utc>>) -> bool {
        if let Some(due_date) = due {
            let now = Utc::now();
            let days_until_due = (due_date.date_naive() - now.date_naive()).num_days();
            days_until_due == 1 // Exactly tomorrow
        } else {
            false
        }
    }
    
    // Helper method to check if task is overdue
    fn is_overdue(&self, due: Option<chrono::DateTime<Utc>>) -> bool {
        if let Some(due_date) = due {
            let now = Utc::now();
            let days_until_due = (due_date.date_naive() - now.date_naive()).num_days();
            days_until_due < 0 // Past due date
        } else {
            false
        }
    }
    
    // ===== FIELD FORMATTERS =====
    
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
            crate::data::models::TaskStatus::Pending => "P".to_string(),
            crate::data::models::TaskStatus::Completed => "C".to_string(),
            crate::data::models::TaskStatus::Deleted => "D".to_string(),
            crate::data::models::TaskStatus::Waiting => "W".to_string(),
            crate::data::models::TaskStatus::Recurring => "R".to_string(),
        }
    }
    
    fn format_priority(&self, priority: &Option<crate::data::models::Priority>) -> String {
        priority.as_ref()
            .map(|p| p.as_char().to_string())
            .unwrap_or_else(|| " ".to_string())
    }
    
    fn format_priority_full(&self, priority: &Option<crate::data::models::Priority>) -> String {
        match priority {
            Some(crate::data::models::Priority::High) => "High".to_string(),
            Some(crate::data::models::Priority::Medium) => "Medium".to_string(),
            Some(crate::data::models::Priority::Low) => "Low".to_string(),
            None => "".to_string(),
        }
    }
    
    fn format_project(&self, project: &Option<String>) -> String {
        project.as_deref()
            .map(|p| if p.len() > 14 { format!("{}...", &p[..11]) } else { p.to_string() })
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
        // Maximum space for description in simplified layout - up to 45+ characters!
        if description.len() > 45 {
            format!("{}...", &description[..42])
        } else {
            description.to_string()
        }
    }
    
    fn format_urgency(&self, urgency: f64) -> String {
        format!("{:.1}", urgency)
    }
}