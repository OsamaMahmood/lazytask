// Dashboard statistics and charts component

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, Cell},
    Frame,
};
use std::collections::HashMap;
use chrono::Utc;

use crate::data::models::{Priority, Task, TaskStatus};

#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub pending: usize,
    pub completed: usize,
    pub deleted: usize,
    pub total: usize,
}

impl ProjectStats {
    pub fn completion_rate(&self) -> f32 {
        let active_total = self.pending + self.completed; // Don't count deleted in completion
        if active_total > 0 {
            self.completed as f32 / active_total as f32 * 100.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskSummaryCache {
    pub total: usize,
    pub pending: usize,
    pub completed: usize,
    pub deleted: usize,
    pub waiting: usize,
    pub active: usize,
    pub overdue: usize,
    pub high_priority: usize,
    pub medium_priority: usize,
    pub low_priority: usize,
    pub no_priority: usize,
    pub avg_urgency: f64,
    pub recent_tasks: usize,
    pub completed_this_week: usize,
    pub version: u64,
}

pub struct DashboardWidget {
    tasks: Vec<Task>,
    project_stats: HashMap<String, ProjectStats>,
    task_summary_cache: Option<TaskSummaryCache>,
}

impl DashboardWidget {
    pub fn new(tasks: Vec<Task>, project_stats: HashMap<String, ProjectStats>, task_summary_cache: Option<TaskSummaryCache>) -> Self {
        DashboardWidget {
            tasks,
            project_stats,
            task_summary_cache,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Responsive reports layout based on terminal size
        let terminal_width = area.width;
        
        if terminal_width < 100 {
            // Narrow screen - vertical stacking (all components)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(25),   // Summary - 25%
                    Constraint::Percentage(40),   // Project table - 40%
                    Constraint::Percentage(35),   // Activity - 35%
                ])
                .split(area);
            
            self.render_enhanced_summary_panel(f, chunks[0]);
            self.render_enhanced_project_table(f, chunks[1]);
            self.render_recent_activity_panel(f, chunks[2]);
        } else {
            // Wide screen - full layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),  // Top row - 50%
                    Constraint::Percentage(50),  // Bottom row - 50%
                ])
                .split(area);

            // Top row: Summary (left) + Burndown (right)
            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(chunks[0]);

            // Bottom row: By Project (left) + Recent Activity (right)
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(chunks[1]);

            self.render_enhanced_summary_panel(f, top_chunks[0]);
            self.render_burndown_panel(f, top_chunks[1]);
            self.render_enhanced_project_table(f, bottom_chunks[0]);
            self.render_recent_activity_panel(f, bottom_chunks[1]);
        }
    }

    fn render_enhanced_summary_panel(&self, f: &mut Frame, area: Rect) {
        let cache = match &self.task_summary_cache {
            Some(cache) => cache,
            None => {
                let empty_text = vec![Line::from("Loading...")];
                let summary = Paragraph::new(empty_text)
                    .block(Block::default().title("Task Summary").borders(Borders::ALL));
                f.render_widget(summary, area);
                return;
            }
        };

        let completion_rate = if cache.total > 0 {
            cache.completed as f32 / cache.total as f32 * 100.0
        } else {
            0.0
        };

        let summary_text = vec![
            Line::from(vec![
                Span::styled("Total Tasks: ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("{}", cache.total), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Pending: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:3} ({:4.1}%)", cache.pending, cache.pending as f32 / cache.total.max(1) as f32 * 100.0)),
            ]),
            Line::from(vec![
                Span::styled("Completed: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{:3} ({:4.1}%)", cache.completed, completion_rate)),
            ]),
            Line::from(vec![
                Span::styled("Deleted: ", Style::default().fg(Color::Red)),
                Span::raw(format!("{:3} ({:4.1}%)", cache.deleted, cache.deleted as f32 / cache.total.max(1) as f32 * 100.0)),
            ]),
            Line::from(vec![
                Span::styled("Waiting: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{:3}", cache.waiting)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Active: ", Style::default().fg(Color::Blue)),
                Span::raw(format!("{}", cache.active)),
            ]),
            Line::from(vec![
                Span::styled("Overdue: ", Style::default().fg(Color::Red)),
                Span::raw(format!("{}", cache.overdue)),
            ]),
        ];

        let summary = Paragraph::new(summary_text)
            .block(Block::default()
                .title("Summary")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)));
        
        f.render_widget(summary, area);
    }

    fn render_burndown_panel(&self, f: &mut Frame, area: Rect) {
        let now = Utc::now();
        let mut daily_counts = vec![0; 30];
        
        for task in &self.tasks {
            if task.status == TaskStatus::Completed {
                if let Some(end_time) = task.end {
                    let days_ago = (now - end_time).num_days();
                    if days_ago >= 0 && days_ago < 30 {
                        let index = (29 - days_ago) as usize;
                        if index < daily_counts.len() {
                            daily_counts[index] += 1;
                        }
                    }
                }
            }
        }
        
        let max_count = *daily_counts.iter().max().unwrap_or(&1).max(&1) as f32;
        
        let mut burndown_lines = vec![Line::from("     │")];
        
        for level in (1..=8).rev() {
            let threshold = (max_count * level as f32 / 8.0) as i32;
            let mut line = format!("{:4} ┤ ", threshold);
            
            for &count in &daily_counts[15..30] {
                if count >= threshold {
                    line.push('●');
                } else {
                    line.push('○');
                }
            }
            
            burndown_lines.push(Line::from(line));
            
            if level > 1 {
                burndown_lines.push(Line::from("     │"));
            }
        }
        
        burndown_lines.push(Line::from("     └─────────────────────────"));

        let burndown_panel = Paragraph::new(burndown_lines)
            .block(Block::default()
                .title("Burndown (Last 30 days)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)));
        
        f.render_widget(burndown_panel, area);
    }

    fn render_enhanced_project_table(&self, f: &mut Frame, area: Rect) {
        let header = Row::new(vec![
            Cell::from("Project").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from("Pending").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Cell::from("Completed").style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Cell::from("%Done").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from("Urgency Avg").style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Cell::from("Next Due").style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]);

        let mut rows = Vec::new();
        
        if !self.project_stats.is_empty() {
            let mut projects: Vec<_> = self.project_stats.iter().collect();
            projects.sort_by(|a, b| (b.1.pending + b.1.completed).cmp(&(a.1.pending + a.1.completed)));

            let max_projects = (area.height.saturating_sub(3) as usize).max(8);

            for (project_name, stats) in projects.into_iter().take(max_projects) {
                let completion_rate = stats.completion_rate();
                
                let project_urgency: f64 = self.tasks.iter()
                    .filter(|t| t.project.as_ref().map(|p| p == project_name).unwrap_or(project_name == "(no project)"))
                    .filter(|t| t.status == TaskStatus::Pending)
                    .map(|t| t.urgency)
                    .sum::<f64>() / stats.pending.max(1) as f64;

                let next_due = self.tasks.iter()
                    .filter(|t| t.project.as_ref().map(|p| p == project_name).unwrap_or(project_name == "(no project)"))
                    .filter(|t| t.status == TaskStatus::Pending && t.due.is_some())
                    .min_by_key(|t| t.due)
                    .and_then(|t| t.due)
                    .map(|due| {
                        let days_until = (due - chrono::Utc::now()).num_days();
                        if days_until < 0 {
                            format!("{}d ago", -days_until)
                        } else if days_until == 0 {
                            "Today".to_string()
                        } else if days_until == 1 {
                            "Tomorrow".to_string()
            } else {
                            format!("{}d", days_until)
                        }
                    })
                    .unwrap_or("-".to_string());

                let row = Row::new(vec![
                    Cell::from(format!("{}", project_name)).style(Style::default().fg(Color::Green)),
                    Cell::from(format!("{}", stats.pending)).style(Style::default().fg(Color::Yellow)),
                    Cell::from(format!("{}", stats.completed)).style(Style::default().fg(Color::Green)),
                    Cell::from(format!("{:.0}%", completion_rate)).style(
                        if completion_rate >= 80.0 { Style::default().fg(Color::Green) }
                        else if completion_rate >= 50.0 { Style::default().fg(Color::Yellow) }
                        else { Style::default().fg(Color::Red) }
                    ),
                    Cell::from(format!("{:.1}", project_urgency)).style(
                        if project_urgency >= 10.0 { Style::default().fg(Color::Red) }
                        else if project_urgency >= 5.0 { Style::default().fg(Color::Yellow) }
                        else { Style::default().fg(Color::Green) }
                    ),
                    Cell::from(next_due.clone()).style(
                        if next_due.contains("ago") || next_due == "Today" { Style::default().fg(Color::Red) }
                        else if next_due == "Tomorrow" { Style::default().fg(Color::Yellow) }
                        else { Style::default().fg(Color::White) }
                    ),
                ]);
                rows.push(row);
            }
        }

        let table = Table::new(rows)
            .header(header)
            .block(Block::default()
                .title("By Project")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)))
            .widths(&[
                Constraint::Length(14),
                Constraint::Length(9),
                Constraint::Length(11),
                Constraint::Length(7),
                Constraint::Length(13),
                Constraint::Length(12),
            ])
            .column_spacing(1);
        
        f.render_widget(table, area);
    }

    fn render_recent_activity_panel(&self, f: &mut Frame, area: Rect) {
        let now = chrono::Utc::now();
        let mut recent_activities = Vec::new();
        
        for task in &self.tasks {
            if task.status == TaskStatus::Completed {
                if let Some(end_time) = task.end {
                    if end_time > now - chrono::Duration::days(7) {
                        let time_ago = now - end_time;
                        let time_str = if time_ago.num_minutes() < 60 {
                            format!("{}min ago", time_ago.num_minutes())
                        } else if time_ago.num_hours() < 24 {
                            format!("{}h ago", time_ago.num_hours())
                        } else {
                            format!("{}d ago", time_ago.num_days())
                        };
                        
                        let activity_type = match &task.project {
                            Some(project) => format!("Completed in [{}]", project),
                            None => "Completed (no project)".to_string(),
                        };
                        recent_activities.push((end_time, activity_type, task.description.clone(), task.project.clone(), time_str));
                    }
                }
            }
            
            if task.entry > now - chrono::Duration::days(3) {
                let time_ago = now - task.entry;
                let time_str = if time_ago.num_minutes() < 60 {
                    format!("{}min ago", time_ago.num_minutes())
                } else if time_ago.num_hours() < 24 {
                    format!("{}h ago", time_ago.num_hours())
                } else {
                    format!("{}d ago", time_ago.num_days())
                };
                
                let priority_suffix = match &task.priority {
                    Some(Priority::High) => " [H]",
                    Some(Priority::Medium) => " [M]", 
                    Some(Priority::Low) => " [L]",
                    None => "",
                };

                let activity_type = match &task.project {
                    Some(project) => {
                        if task.tags.is_empty() {
                            if task.due.is_some() {
                                format!("Task+due added to [{}]{}", project, priority_suffix)
                            } else {
                                format!("Task added to [{}]{}", project, priority_suffix)
                            }
                        } else {
                            if task.due.is_some() {
                                format!("Task+tags+due added to [{}]{}", project, priority_suffix)
                            } else {
                                format!("Task+tags added to [{}]{}", project, priority_suffix)
                            }
                        }
                    },
                    None => {
                        if task.tags.is_empty() {
                            if task.due.is_some() {
                                format!("Task+due added{}", priority_suffix)
                            } else {
                                format!("Task added{}", priority_suffix)
                            }
                        } else {
                            if task.due.is_some() {
                                format!("Task+tags+due added{}", priority_suffix)
                            } else {
                                format!("Task+tags added{}", priority_suffix)
                            }
                        }
                    }
                };
                recent_activities.push((task.entry, activity_type, task.description.clone(), task.project.clone(), time_str));
            }
        }
        
        recent_activities.sort_by(|a, b| b.0.cmp(&a.0));
        
        let max_items = (area.height.saturating_sub(2) as usize).max(6);
        recent_activities.truncate(max_items);
        
        let mut activity_text = vec![];
        
        if recent_activities.is_empty() {
            activity_text.push(Line::from("No recent activity"));
        } else {
            for (_, action, description, _project, time_str) in recent_activities {
                let short_desc = if description.len() > 45 {
                    format!("{}...", &description[..42])
                } else {
                    description
                };
                
                let action_color = if action.contains("Completed") {
                    Color::Green
                } else if action.contains("[H]") {
                    Color::Red
                } else if action.contains("[M]") {
                    Color::Yellow
                } else if action.contains("tags") {
                    Color::Magenta
                } else {
                    Color::Blue
                };
                
                activity_text.push(Line::from(vec![
                    Span::styled(format!("{:8} ", time_str), Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{:30} ", action), Style::default().fg(action_color)),
                    Span::styled(short_desc, Style::default().fg(Color::White)),
                ]));
            }
        }

        let activity_panel = Paragraph::new(activity_text)
            .block(Block::default()
                .title("Recent Activity")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)));
        
        f.render_widget(activity_panel, area);
    }
}
