// Reports dashboard view

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, Cell},
    Frame,
};
use std::collections::HashMap;
use chrono::{Duration, Utc};

use crate::data::models::{Priority, Task, TaskStatus};

#[derive(Debug, Clone)]
struct ProjectStats {
    pending: usize,
    completed: usize,
    deleted: usize,
    total: usize,
}

impl ProjectStats {
    fn completion_rate(&self) -> f32 {
        let active_total = self.pending + self.completed; // Don't count deleted in completion
        if active_total > 0 {
            self.completed as f32 / active_total as f32 * 100.0
        } else {
            0.0
        }
    }
}

pub struct ReportsView {
    tasks: Vec<Task>,
    // Cache expensive calculations
    project_stats: HashMap<String, ProjectStats>,
    task_summary_cache: Option<TaskSummaryCache>,
    data_version: u64, // Track when data changes
}

#[derive(Debug, Clone)]
struct TaskSummaryCache {
    total: usize,
    pending: usize,
    completed: usize,
    deleted: usize,
    waiting: usize,
    active: usize,
    overdue: usize,
    high_priority: usize,
    medium_priority: usize,
    low_priority: usize,
    no_priority: usize,
    avg_urgency: f64,
    recent_tasks: usize,
    completed_this_week: usize,
    version: u64,
}

impl ReportsView {
    pub fn new() -> Self {
        ReportsView {
            tasks: Vec::new(),
            project_stats: HashMap::new(),
            task_summary_cache: None,
            data_version: 0,
        }
    }

    pub fn update_tasks(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
        self.data_version += 1; // Increment version to invalidate cache
        self.recalculate_stats();
    }

    fn recalculate_stats(&mut self) {
        // Recalculate project statistics
        self.project_stats.clear();
        
        for task in &self.tasks {
            let project_name = task.project.clone().unwrap_or_else(|| "(no project)".to_string());
            let stats = self.project_stats.entry(project_name).or_insert(ProjectStats {
                pending: 0,
                completed: 0,
                deleted: 0,
                total: 0,
            });
            
            match task.status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::Completed => stats.completed += 1,
                TaskStatus::Deleted => stats.deleted += 1,
                TaskStatus::Waiting => stats.pending += 1, // Count waiting as pending for stats
                TaskStatus::Recurring => stats.pending += 1, // Count recurring as pending for stats
            }
            stats.total += 1;
        }

        // Recalculate summary cache
        self.calculate_summary_cache();
    }

    fn calculate_summary_cache(&mut self) {
        let total = self.tasks.len();
        let pending = self.tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();
        let completed = self.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let deleted = self.tasks.iter().filter(|t| t.status == TaskStatus::Deleted).count();
        let waiting = self.tasks.iter().filter(|t| t.status == TaskStatus::Waiting).count();
        let active = self.tasks.iter().filter(|t| t.is_active()).count();
        let overdue = self.tasks.iter().filter(|t| t.is_overdue()).count();

        let high_priority = self.tasks.iter().filter(|t| t.priority == Some(Priority::High)).count();
        let medium_priority = self.tasks.iter().filter(|t| t.priority == Some(Priority::Medium)).count();
        let low_priority = self.tasks.iter().filter(|t| t.priority == Some(Priority::Low)).count();
        let no_priority = self.tasks.iter().filter(|t| t.priority.is_none()).count();
        
        let avg_urgency = if !self.tasks.is_empty() {
            self.tasks.iter().map(|t| t.urgency).sum::<f64>() / self.tasks.len() as f64
        } else {
            0.0
        };

        // Calculate recent activity
        use chrono::{Duration, Utc};
        let now = Utc::now();
        let week_ago = now - Duration::days(7);
        
        let recent_tasks = self.tasks.iter()
            .filter(|t| t.entry > week_ago)
            .count();
        
        let completed_this_week = self.tasks.iter()
            .filter(|t| t.status == TaskStatus::Completed && 
                        t.end.map_or(false, |end| end > week_ago))
            .count();

        self.task_summary_cache = Some(TaskSummaryCache {
            total,
            pending,
            completed,
            deleted,
            waiting,
            active,
            overdue,
            high_priority,
            medium_priority,
            low_priority,
            no_priority,
            avg_urgency,
            recent_tasks,
            completed_this_week,
            version: self.data_version,
        });
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Responsive reports layout based on terminal size
        // Always show all components, maximize space utilization
        let terminal_width = area.width;
        let terminal_height = area.height;
        
        if terminal_width < 100 {
            // Narrow screen - vertical stacking (all components)
            // Ensure all sections are always visible with flexible sizing
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
            // Wide screen - full layout with Summary + Burndown side by side
            // Maximize space usage - fill entire window
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),  // Top row - 50% of height (Summary + Burndown)
                    Constraint::Percentage(50),  // Bottom row - 50% of height (By Project + Recent Activity)
                ])
                .split(area);

            // Top row: Summary (left) + Burndown (right)
            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50), // Summary
                    Constraint::Percentage(50), // Burndown chart
                ])
                .split(chunks[0]);

            // Bottom row: By Project (left) + Recent Activity (right)
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50), // By Project
                    Constraint::Percentage(50), // Recent Activity
                ])
                .split(chunks[1]);

            // Render all enhanced report panels
            self.render_enhanced_summary_panel(f, top_chunks[0]);
            self.render_burndown_panel(f, top_chunks[1]);
            self.render_enhanced_project_table(f, bottom_chunks[0]);
            self.render_recent_activity_panel(f, bottom_chunks[1]);
        }
    }

    fn render_enhanced_summary_panel(&self, f: &mut Frame, area: Rect) {
        // Use cached data if available
        let cache = match &self.task_summary_cache {
            Some(cache) => cache,
            None => {
                // Fallback to empty data if cache not available
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
        // Calculate actual burndown data for last 30 days
        let now = Utc::now();
        let mut daily_counts = vec![0; 30];
        
        // Count tasks completed each day for the last 30 days
        for task in &self.tasks {
            if task.status == TaskStatus::Completed {
                if let Some(end_time) = task.end {
                    let days_ago = (now - end_time).num_days();
                    if days_ago >= 0 && days_ago < 30 {
                        let index = (29 - days_ago) as usize; // Reverse order for left-to-right display
                        if index < daily_counts.len() {
                            daily_counts[index] += 1;
                        }
                    }
                }
            }
        }
        
        // Find the maximum count for scaling
        let max_count = *daily_counts.iter().max().unwrap_or(&1).max(&1) as f32;
        
        // Create visual burndown chart
        let mut burndown_lines = vec![
            Line::from("     │"),
        ];
        
        // Create 8 levels for the chart (from 20 down to 5)
        for level in (1..=8).rev() {
            let threshold = (max_count * level as f32 / 8.0) as i32;
            let mut line = format!("{:4} ┤ ", threshold);
            
            for &count in &daily_counts[15..30] { // Show last 15 days for better fit
                if count >= threshold {
                    line.push('●');
                } else {
                    line.push('○');
                }
            }
            
            burndown_lines.push(Line::from(line));
            
            // Add spacing line for readability
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
        // Create a proper table matching the plan.md layout
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
            // Sort projects by total activity (pending + completed)
            let mut projects: Vec<_> = self.project_stats.iter().collect();
            projects.sort_by(|a, b| (b.1.pending + b.1.completed).cmp(&(a.1.pending + a.1.completed)));

            // Dynamically calculate max projects based on available height
            // Subtract 3 for borders (2) and header (1), minimum 8 projects
            let max_projects = (area.height.saturating_sub(3) as usize).max(8);

            for (project_name, stats) in projects.into_iter().take(max_projects) {
                let completion_rate = stats.completion_rate();
                
                // Calculate average urgency for this project
                let project_urgency: f64 = self.tasks.iter()
                    .filter(|t| t.project.as_ref().map(|p| p == project_name).unwrap_or(project_name == "(no project)"))
                    .filter(|t| t.status == TaskStatus::Pending)
                    .map(|t| t.urgency)
                    .sum::<f64>() / stats.pending.max(1) as f64;

                // Find next due date for this project
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
                Constraint::Length(14), // Project
                Constraint::Length(9),  // Pending
                Constraint::Length(11), // Completed
                Constraint::Length(7),  // %Done
                Constraint::Length(13), // Urgency Avg
                Constraint::Length(12), // Next Due
            ])
            .column_spacing(1);
        
        f.render_widget(table, area);
    }

    fn render_recent_activity_panel(&self, f: &mut Frame, area: Rect) {
        
        // Get recent activity with detailed timestamps
        let now = chrono::Utc::now();
        
        // Find recently completed tasks (last 7 days)
        let mut recent_activities = Vec::new();
        
        for task in &self.tasks {
            // Check for recent completions
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
            
            // Check for recent additions (last 3 days for less noise)
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
                    Some(crate::data::models::Priority::High) => " [H]",
                    Some(crate::data::models::Priority::Medium) => " [M]", 
                    Some(crate::data::models::Priority::Low) => " [L]",
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
        
        // Sort by time (most recent first)
        recent_activities.sort_by(|a, b| b.0.cmp(&a.0));
        
        // Dynamically calculate max items based on available height
        // Subtract 2 for borders, then show as many as fit
        let max_items = (area.height.saturating_sub(2) as usize).max(6);
        recent_activities.truncate(max_items);
        
        let mut activity_text = vec![];
        
        if recent_activities.is_empty() {
            activity_text.push(Line::from("No recent activity"));
        } else {
            for (_, action, description, _project, time_str) in recent_activities {
                // Truncate long descriptions
                let short_desc = if description.len() > 45 {
                    format!("{}...", &description[..42])
                } else {
                    description
                };
                
                // Color code by activity type
                let action_color = if action.contains("Completed") {
                    Color::Green
                } else if action.contains("[H]") {
                    Color::Red      // High priority
                } else if action.contains("[M]") {
                    Color::Yellow   // Medium priority
                } else if action.contains("tags") {
                    Color::Magenta  // Has tags
                } else {
                    Color::Blue     // Regular add
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
