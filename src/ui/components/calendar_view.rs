// Calendar widget component with daily stats

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use crate::data::models::{Priority, Task, TaskStatus};

pub struct CalendarWidget {
    pub selected_date: DateTime<Utc>,
    pub tasks: Vec<Task>,
}

impl CalendarWidget {
    pub fn new(selected_date: DateTime<Utc>, tasks: Vec<Task>) -> Self {
        CalendarWidget {
            selected_date,
            tasks,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Split area: 3-Month Calendar grid (top) + Daily stats (bottom)
        // Give more space to calendar now that we have 3 months
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // 3-month calendar grid (60%)
                Constraint::Percentage(60), // Daily stats (40%)
            ])
            .split(area);

        self.render_calendar_grid(f, chunks[0]);
        self.render_daily_stats(f, chunks[1]);
    }

    fn get_tasks_for_date(&self, date: DateTime<Utc>) -> Vec<&Task> {
        let target_date = date.date_naive();
        
        self.tasks.iter().filter(|task| {
            // Include tasks with due date on this day
            let has_due_date = task.due.map_or(false, |due| due.date_naive() == target_date);
            
            // Include tasks completed on this day
            let completed_on_date = task.end.map_or(false, |end| end.date_naive() == target_date);
            
            // Include tasks created on this day
            let created_on_date = task.entry.date_naive() == target_date;
            
            has_due_date || completed_on_date || created_on_date
        }).collect()
    }

    fn render_calendar_grid(&self, f: &mut Frame, area: Rect) {
        // Calculate the 3 months to display (previous, current, next)
        let center_date = self.selected_date;
        
        // Create horizontal layout for 3 months
        let month_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33), // Previous month
                Constraint::Percentage(34), // Current month (slightly larger)
                Constraint::Percentage(33), // Next month
            ])
            .split(area);
        
        // Render each month
        self.render_single_month(f, month_chunks[0], center_date, -1);
        self.render_single_month(f, month_chunks[1], center_date, 0);
        self.render_single_month(f, month_chunks[2], center_date, 1);
    }
    
    fn render_single_month(&self, f: &mut Frame, area: Rect, center_date: DateTime<Utc>, month_offset: i32) {
        // Calculate the target month based on offset
        let target_date = if month_offset < 0 {
            center_date - Duration::days(30 * month_offset.abs() as i64)
        } else if month_offset > 0 {
            center_date + Duration::days(30 * month_offset as i64)
        } else {
            center_date
        };
        
        let selected_year = self.selected_date.year();
        let selected_month = self.selected_date.month();
        let selected_day = self.selected_date.day();
        
        let target_year = target_date.year();
        let target_month = target_date.month();
        
        // Get first day of target month
        let first_day = NaiveDate::from_ymd_opt(target_year, target_month, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        
        // Get last day of target month
        let days_in_month = if target_month == 12 {
            NaiveDate::from_ymd_opt(target_year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(target_year, target_month + 1, 1)
        }.unwrap().pred_opt().unwrap().day();

        // Build calendar
        let month_name = match target_month {
            1 => "January", 2 => "February", 3 => "March", 4 => "April",
            5 => "May", 6 => "June", 7 => "July", 8 => "August",
            9 => "September", 10 => "October", 11 => "November", 12 => "December",
            _ => "Unknown",
        };
        
        // Shorter title for 3-month view
        let title = if area.width < 35 {
            format!("{} '{:02}", &month_name[..3], target_year % 100) // "Oct '25"
        } else {
            format!("{} {}", month_name, target_year)
        };
        
        // Make center month title more prominent
        let title_display = if month_offset == 0 {
            // Center month: Add decorative markers and make it stand out
            format!("═══ {} ═══", title)
        } else {
            // Side months: Keep simple
            title
        };
        
        // Calculate dynamic width for title
        let title_width = area.width.saturating_sub(2) as usize;
        
        // Different styling for center vs side month titles
        let title_style = if month_offset == 0 {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED)  // Underline for emphasis
        } else {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        };
        
        let mut calendar_text = vec![
            Line::from(vec![
                Span::styled(format!("{:^width$}", title_display, width = title_width), title_style)
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Mo   ", Style::default().fg(Color::Yellow)),
                Span::styled("   Tu   ", Style::default().fg(Color::Yellow)),
                Span::styled("   We   ", Style::default().fg(Color::Yellow)),
                Span::styled("   Th   ", Style::default().fg(Color::Yellow)),
                Span::styled("   Fr   ", Style::default().fg(Color::Yellow)),
                Span::styled("   Sa   ", Style::default().fg(Color::Cyan)),
                Span::styled("   Su   ", Style::default().fg(Color::Cyan)),
            ]),
        ];

        // Calculate starting day of week (0 = Monday, 6 = Sunday)
        let start_weekday = first_day.weekday().num_days_from_monday();
        
        // Build week rows
        let mut current_day = 1;
        let mut current_weekday = start_weekday;
        
        while current_day <= days_in_month {
            let mut week_line = Vec::new();
            
            for _ in 0..7 {
                if current_weekday < start_weekday && current_day == 1 {
                    // Empty day before month starts - match header width (8 chars)
                    week_line.push(Span::raw("        "));
                    current_weekday += 1;
                } else if current_day > days_in_month {
                    // Empty day after month ends - match header width (8 chars)
                    week_line.push(Span::raw("        "));
                } else {
                    // Actual day
                    let date = NaiveDate::from_ymd_opt(target_year, target_month, current_day)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_utc();
                    
                    let tasks_on_day = self.get_tasks_for_date(date);
                    let task_count = tasks_on_day.len();
                    
                    // Determine task indicators
                    let (indicator, indicator_color) = if task_count == 0 {
                        ("  ", Color::White)
                    } else {
                        let has_overdue = tasks_on_day.iter().any(|t| t.is_overdue());
                        let has_pending = tasks_on_day.iter().any(|t| t.status == TaskStatus::Pending);
                        let all_completed = tasks_on_day.iter().all(|t| t.status == TaskStatus::Completed);
                        
                        if has_overdue {
                            ("⚠", Color::Red)
                        } else if all_completed {
                            ("✓", Color::Green)
                        } else if has_pending {
                            ("•", Color::Yellow)
                        } else {
                            ("○", Color::Cyan)
                        }
                    };
                    
                    // Check if this day is the selected date (must match month/year too)
                    let is_selected = current_day == selected_day && 
                                    target_year == selected_year && 
                                    target_month == selected_month;
                    
                    let is_today = {
                        let today = Utc::now();
                        today.year() == target_year && 
                        today.month() == target_month && 
                        today.day() == current_day
                    };
                    
                    // Format: "   DD   " (8 chars) with optional indicator
                    // Always make date numbers BOLD for readability
                    
                    let mut style = if is_selected {
                        Style::default().fg(Color::Black).bg(Color::Yellow)
                    } else if is_today {
                        Style::default().fg(Color::Cyan)
                    } else if task_count > 0 {
                        Style::default().fg(indicator_color)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    
                    // Always make date numbers bold
                    style = style.add_modifier(Modifier::BOLD);
                    
                    // Always use 8-character width to match header
                    if task_count > 0 && area.width >= 30 {
                        // Format: "   DD·  " where · is the indicator (8 chars total)
                        week_line.push(Span::styled(
                            format!("   {:>2}{}  ", current_day, indicator), 
                            style
                        ));
                    } else {
                        // Format: "   DD   " (8 chars, centered)
                        week_line.push(Span::styled(
                            format!("   {:>2}   ", current_day), 
                            style
                        ));
                    }
                    
                    current_day += 1;
                }
            }
            
            // Add the week row
            calendar_text.push(Line::from(week_line));
            
            // Add vertical spacing (blank line) between weeks for better readability
            // Don't add after the last week to save space
            if current_day <= days_in_month {
                calendar_text.push(Line::from(""));
            }
        }
        
        // Only show legend on the center month (month_offset == 0) - CENTERED
        if month_offset == 0 && area.height > 12 {
            calendar_text.push(Line::from(""));
            if area.width > 40 {
                // Create centered legend line
                let legend_spans = vec![
                    Span::styled("⚠", Style::default().fg(Color::Red)),
                    Span::raw("=Overdue  "),
                    Span::styled("•", Style::default().fg(Color::Yellow)),
                    Span::raw("=Pending  "),
                    Span::styled("✓", Style::default().fg(Color::Green)),
                    Span::raw("=Done"),
                ];
                
                // Calculate padding for centering
                let legend_text_width = 32; // Approximate width of legend text
                let padding = ((area.width.saturating_sub(2) as usize).saturating_sub(legend_text_width)) / 2;
                let padding_str = " ".repeat(padding);
                
                // Add padding before legend to center it
                let mut centered_line = vec![Span::raw(padding_str)];
                centered_line.extend(legend_spans);
                
                calendar_text.push(Line::from(centered_line));
            }
        }

        // Use different border styles for center vs side months
        let border_type = if month_offset == 0 {
            ratatui::widgets::BorderType::Double  // Prominent double border for active month
        } else {
            ratatui::widgets::BorderType::Plain    // Normal border for side months
        };

        let calendar = Paragraph::new(calendar_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(border_type)
                .border_style(Style::default().fg(
                    if month_offset == 0 { 
                        Color::Cyan // Highlight current month
                    } else { 
                        Color::DarkGray // Dim previous/next months
                    }
                )));
        
        f.render_widget(calendar, area);
    }

    fn render_daily_stats(&self, f: &mut Frame, area: Rect) {
        let tasks_on_day = self.get_tasks_for_date(self.selected_date);
        
        let date_str = self.selected_date.format("%A, %B %d, %Y").to_string();
        
        // Calculate stats
        let total_tasks = tasks_on_day.len();
        let pending = tasks_on_day.iter().filter(|t| t.status == TaskStatus::Pending).count();
        let completed = tasks_on_day.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let deleted = tasks_on_day.iter().filter(|t| t.status == TaskStatus::Deleted).count();
        let overdue = tasks_on_day.iter().filter(|t| t.is_overdue()).count();
        
        let with_due_date = tasks_on_day.iter().filter(|t| {
            t.due.map_or(false, |due| due.date_naive() == self.selected_date.date_naive())
        }).count();
        let completed_on_date = tasks_on_day.iter().filter(|t| {
            t.end.map_or(false, |end| end.date_naive() == self.selected_date.date_naive())
        }).count();
        let created_on_date = tasks_on_day.iter().filter(|t| {
            t.entry.date_naive() == self.selected_date.date_naive()
        }).count();
        
        let avg_urgency = if !tasks_on_day.is_empty() {
            tasks_on_day.iter().map(|t| t.urgency).sum::<f64>() / tasks_on_day.len() as f64
        } else {
            0.0
        };
        
        // Build stats text
        let mut stats_text = vec![
            Line::from(vec![
                Span::styled(format!("📅 {}", date_str), 
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("📊 Daily Summary:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("  Total tasks: "),
                Span::styled(format!("{}", total_tasks), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("•", Style::default().fg(Color::Yellow)),
                Span::raw(" Pending: "),
                Span::styled(format!("{}", pending), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("✓", Style::default().fg(Color::Green)),
                Span::raw(" Completed: "),
                Span::styled(format!("{}", completed), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("✗", Style::default().fg(Color::Red)),
                Span::raw(" Deleted: "),
                Span::styled(format!("{}", deleted), Style::default().fg(Color::Red)),
            ]),
        ];
        
        if overdue > 0 {
            stats_text.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("⚠️", Style::default().fg(Color::Red)),
                Span::raw(" Overdue: "),
                Span::styled(format!("{}", overdue), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            ]));
        }
        
        stats_text.push(Line::from(""));
        stats_text.push(Line::from(vec![
            Span::styled("📋 Task Categories:", Style::default().fg(Color::Cyan)),
        ]));
        stats_text.push(Line::from(vec![
            Span::raw(format!("  Due today: {} | Completed today: {} | Created today: {}", 
                with_due_date, completed_on_date, created_on_date)),
        ]));
        
        if !tasks_on_day.is_empty() {
            stats_text.push(Line::from(vec![
                Span::raw(format!("  Average urgency: ")),
                Span::styled(format!("{:.1}", avg_urgency), 
                    if avg_urgency >= 10.0 { Style::default().fg(Color::Red) }
                    else if avg_urgency >= 5.0 { Style::default().fg(Color::Yellow) }
                    else { Style::default().fg(Color::Green) }
                ),
            ]));
        }
        
        // List tasks
        if !tasks_on_day.is_empty() {
            stats_text.push(Line::from(""));
            stats_text.push(Line::from(vec![
                Span::styled("📝 Tasks:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
            
            let max_tasks = (area.height as usize).saturating_sub(stats_text.len() + 3).min(tasks_on_day.len());
            
            for task in tasks_on_day.iter().take(max_tasks) {
                let status_icon = match task.status {
                    TaskStatus::Pending => if task.is_overdue() { "⚠️" } else { "•" },
                    TaskStatus::Completed => "✓",
                    TaskStatus::Deleted => "✗",
                    TaskStatus::Waiting => "⏸",
                    TaskStatus::Recurring => "🔁",
                };
                
                let status_color = match task.status {
                    TaskStatus::Pending => if task.is_overdue() { Color::Red } else { Color::Yellow },
                    TaskStatus::Completed => Color::Green,
                    TaskStatus::Deleted => Color::Gray,
                    TaskStatus::Waiting => Color::Cyan,
                    TaskStatus::Recurring => Color::Magenta,
                };
                
                let priority_str = match &task.priority {
                    Some(Priority::High) => " (H)",
                    Some(Priority::Medium) => " (M)",
                    Some(Priority::Low) => " (L)",
                    None => "",
                };
                
                let description = if task.description.len() > 50 {
                    format!("{}...", &task.description[..47])
                } else {
                    task.description.clone()
                };
                
                stats_text.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(status_icon, Style::default().fg(status_color)),
                    Span::raw(" "),
                    Span::raw(description),
                    Span::styled(priority_str, Style::default().fg(Color::Magenta)),
                ]));
            }
            
            if tasks_on_day.len() > max_tasks {
                stats_text.push(Line::from(vec![
                    Span::styled(format!("  ... and {} more", tasks_on_day.len() - max_tasks),
                        Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))
                ]));
            }
        } else {
            stats_text.push(Line::from(""));
            stats_text.push(Line::from(vec![
                Span::styled("No tasks on this date", 
                    Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))
            ]));
        }
        
        let stats_panel = Paragraph::new(stats_text)
            .block(Block::default()
                .title("Daily Details")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)));
        
        f.render_widget(stats_panel, area);
    }
}
