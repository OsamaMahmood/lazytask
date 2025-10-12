// Reports dashboard view

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BarChart, Paragraph, Gauge},
    Frame,
};

use crate::data::models::{Priority, Task, TaskStatus};

pub struct ReportsView {
    tasks: Vec<Task>,
}

impl ReportsView {
    pub fn new() -> Self {
        ReportsView {
            tasks: Vec::new(),
        }
    }

    pub fn update_tasks(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create 2x2 grid layout for different report panels
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Top row
                Constraint::Percentage(50), // Bottom row
            ])
            .split(area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Summary
                Constraint::Percentage(50), // Priority breakdown
            ])
            .split(chunks[0]);

        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Project stats
                Constraint::Percentage(50), // Activity timeline
            ])
            .split(chunks[1]);

        // Render each report panel
        self.render_summary_panel(f, top_chunks[0]);
        self.render_priority_panel(f, top_chunks[1]);
        self.render_project_panel(f, bottom_chunks[0]);
        self.render_activity_panel(f, bottom_chunks[1]);
    }

    fn render_summary_panel(&self, f: &mut Frame, area: Rect) {
        let total = self.tasks.len();
        let pending = self.tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
        let completed = self.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let deleted = self.tasks.iter().filter(|t| t.status == TaskStatus::Deleted).count();
        let waiting = self.tasks.iter().filter(|t| t.status == TaskStatus::Waiting).count();
        let active = self.tasks.iter().filter(|t| t.is_active()).count();
        let overdue = self.tasks.iter().filter(|t| t.is_overdue()).count();

        let completion_rate = if total > 0 {
            (completed as f32 / total as f32 * 100.0)
        } else {
            0.0
        };

        let summary_text = vec![
            Line::from(vec![
                Span::styled("Total Tasks: ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("{}", total), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Pending: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:3} ({:4.1}%)", pending, pending as f32 / total.max(1) as f32 * 100.0)),
            ]),
            Line::from(vec![
                Span::styled("Completed: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{:3} ({:4.1}%)", completed, completion_rate)),
            ]),
            Line::from(vec![
                Span::styled("Deleted: ", Style::default().fg(Color::Red)),
                Span::raw(format!("{:3} ({:4.1}%)", deleted, deleted as f32 / total.max(1) as f32 * 100.0)),
            ]),
            Line::from(vec![
                Span::styled("Waiting: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{:3}", waiting)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Active: ", Style::default().fg(Color::Blue)),
                Span::raw(format!("{}", active)),
            ]),
            Line::from(vec![
                Span::styled("Overdue: ", Style::default().fg(Color::Red)),
                Span::raw(format!("{}", overdue)),
            ]),
        ];

        let summary = Paragraph::new(summary_text)
            .block(Block::default().title("Task Summary").borders(Borders::ALL));
        
        f.render_widget(summary, area);
    }

    fn render_priority_panel(&self, f: &mut Frame, area: Rect) {
        let high_count = self.tasks.iter().filter(|t| t.priority == Some(Priority::High)).count();
        let medium_count = self.tasks.iter().filter(|t| t.priority == Some(Priority::Medium)).count();
        let low_count = self.tasks.iter().filter(|t| t.priority == Some(Priority::Low)).count();
        let no_priority = self.tasks.iter().filter(|t| t.priority.is_none()).count();

        let total_with_priority = high_count + medium_count + low_count;
        let avg_urgency = if !self.tasks.is_empty() {
            self.tasks.iter().map(|t| t.urgency).sum::<f64>() / self.tasks.len() as f64
        } else {
            0.0
        };

        let priority_text = vec![
            Line::from(vec![
                Span::styled("Priority Distribution:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("High: ", Style::default().fg(Color::Red)),
                Span::raw(format!("{:2} tasks", high_count)),
            ]),
            Line::from(vec![
                Span::styled("Medium: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:2} tasks", medium_count)),
            ]),
            Line::from(vec![
                Span::styled("Low: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{:2} tasks", low_count)),
            ]),
            Line::from(vec![
                Span::styled("None: ", Style::default().fg(Color::Gray)),
                Span::raw(format!("{:2} tasks", no_priority)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Avg Urgency: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{:.2}", avg_urgency)),
            ]),
        ];

        let priority_panel = Paragraph::new(priority_text)
            .block(Block::default().title("Priority Analysis").borders(Borders::ALL));
        
        f.render_widget(priority_panel, area);
    }

    fn render_project_panel(&self, f: &mut Frame, area: Rect) {
        use std::collections::HashMap;
        
        let mut project_counts: HashMap<String, (usize, usize)> = HashMap::new();
        
        for task in &self.tasks {
            let project_name = task.project.clone().unwrap_or_else(|| "(no project)".to_string());
            let entry = project_counts.entry(project_name).or_insert((0, 0));
            
            match task.status {
                TaskStatus::Pending => entry.0 += 1,
                TaskStatus::Completed => entry.1 += 1,
                _ => {}
            }
        }

        let mut project_lines = vec![
            Line::from(vec![
                Span::styled("Projects Overview:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
        ];

        let mut projects: Vec<_> = project_counts.into_iter().collect();
        projects.sort_by(|a, b| (a.1.0 + a.1.1).cmp(&(b.1.0 + b.1.1)).reverse());

        for (project, (pending, completed)) in projects.into_iter().take(8) {
            let total = pending + completed;
            let completion_rate = if total > 0 {
                completed as f32 / total as f32 * 100.0
            } else {
                0.0
            };
            
            project_lines.push(Line::from(vec![
                Span::styled(format!("{:12}", project), Style::default().fg(Color::Green)),
                Span::raw(format!(" {:2}p ", pending)),
                Span::styled(format!("{:2}c", completed), Style::default().fg(Color::Green)),
                Span::raw(format!(" ({:4.1}%)", completion_rate)),
            ]));
        }

        let project_panel = Paragraph::new(project_lines)
            .block(Block::default().title("Project Status").borders(Borders::ALL));
        
        f.render_widget(project_panel, area);
    }

    fn render_activity_panel(&self, f: &mut Frame, area: Rect) {
        use chrono::{Duration, Utc};
        
        // Calculate recent activity (last 7 days)
        let now = Utc::now();
        let week_ago = now - Duration::days(7);
        
        let recent_tasks = self.tasks.iter()
            .filter(|t| t.entry > week_ago)
            .count();
        
        let completed_this_week = self.tasks.iter()
            .filter(|t| t.status == TaskStatus::Completed && 
                        t.end.map_or(false, |end| end > week_ago))
            .count();

        let active_tasks = self.tasks.iter()
            .filter(|t| t.is_active())
            .count();

        let activity_text = vec![
            Line::from(vec![
                Span::styled("Recent Activity (7 days):", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Added: ", Style::default().fg(Color::Blue)),
                Span::raw(format!("{} tasks", recent_tasks)),
            ]),
            Line::from(vec![
                Span::styled("Completed: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{} tasks", completed_this_week)),
            ]),
            Line::from(vec![
                Span::styled("Currently Active: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{} tasks", active_tasks)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Productivity: ", Style::default().fg(Color::Cyan)),
                Span::raw(if completed_this_week > 0 { 
                    "🟢 Good pace" 
                } else if recent_tasks > 5 {
                    "🟡 Adding many tasks"
                } else {
                    "🔵 Steady state"
                }),
            ]),
        ];

        let activity_panel = Paragraph::new(activity_text)
            .block(Block::default().title("Activity Trends").borders(Borders::ALL));
        
        f.render_widget(activity_panel, area);
    }
}
