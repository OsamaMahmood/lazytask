// Reports view - coordinates between dashboard and calendar modes

use ratatui::Frame;
use ratatui::layout::Rect;
use std::collections::HashMap;
use chrono::{DateTime, Datelike, Duration, Utc};

use crate::data::models::{Priority, Task, TaskStatus};
use crate::ui::components::calendar_view::CalendarWidget;
use crate::ui::components::report_panel::{DashboardWidget, ProjectStats, TaskSummaryCache};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReportMode {
    Dashboard,  // Statistics dashboard
    Calendar,   // Calendar view
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateNavigation {
    NextDay,
    PrevDay,
    NextWeek,
    PrevWeek,
    NextMonth,
    PrevMonth,
    Today,
}


pub struct ReportsView {
    tasks: Vec<Task>,
    // Cache expensive calculations
    project_stats: HashMap<String, ProjectStats>,
    task_summary_cache: Option<TaskSummaryCache>,
    data_version: u64, // Track when data changes
    // Calendar mode state
    mode: ReportMode,
    selected_date: DateTime<Utc>,
}

impl ReportsView {
    pub fn new() -> Self {
        ReportsView {
            tasks: Vec::new(),
            project_stats: HashMap::new(),
            task_summary_cache: None,
            data_version: 0,
            mode: ReportMode::Dashboard,
            selected_date: Utc::now(),
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

    // Calendar mode methods
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            ReportMode::Dashboard => ReportMode::Calendar,
            ReportMode::Calendar => ReportMode::Dashboard,
        };
    }

    pub fn is_calendar_mode(&self) -> bool {
        self.mode == ReportMode::Calendar
    }

    pub fn navigate_date(&mut self, direction: DateNavigation) {
        match direction {
            DateNavigation::NextDay => {
                self.selected_date = self.selected_date + Duration::days(1);
            }
            DateNavigation::PrevDay => {
                self.selected_date = self.selected_date - Duration::days(1);
            }
            DateNavigation::NextWeek => {
                self.selected_date = self.selected_date + Duration::weeks(1);
            }
            DateNavigation::PrevWeek => {
                self.selected_date = self.selected_date - Duration::weeks(1);
            }
            DateNavigation::NextMonth => {
                // Proper month arithmetic: go to same day next month
                let current = self.selected_date;
                let next_month = if current.month() == 12 {
                    chrono::NaiveDate::from_ymd_opt(current.year() + 1, 1, current.day().min(31))
                } else {
                    chrono::NaiveDate::from_ymd_opt(current.year(), current.month() + 1, current.day().min(31))
                };
                
                if let Some(date) = next_month {
                    self.selected_date = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                } else {
                    // Fallback to adding days if date calculation fails
                    self.selected_date = self.selected_date + Duration::days(30);
                }
            }
            DateNavigation::PrevMonth => {
                // Proper month arithmetic: go to same day previous month
                let current = self.selected_date;
                let prev_month = if current.month() == 1 {
                    chrono::NaiveDate::from_ymd_opt(current.year() - 1, 12, current.day().min(31))
                } else {
                    chrono::NaiveDate::from_ymd_opt(current.year(), current.month() - 1, current.day().min(31))
                };
                
                if let Some(date) = prev_month {
                    self.selected_date = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                } else {
                    // Fallback to subtracting days if date calculation fails
                    self.selected_date = self.selected_date - Duration::days(30);
                }
            }
            DateNavigation::Today => {
                self.selected_date = Utc::now();
            }
        }
    }


    pub fn render(&self, f: &mut Frame, area: Rect) {
        match self.mode {
            ReportMode::Dashboard => {
                // Delegate dashboard rendering to DashboardWidget
                let dashboard = DashboardWidget::new(
                    self.tasks.clone(),
                    self.project_stats.clone(),
                    self.task_summary_cache.clone()
                );
                dashboard.render(f, area);
            }
            ReportMode::Calendar => self.render_calendar(f, area),
        }
    }


    fn render_calendar(&self, f: &mut Frame, area: Rect) {
        // Use CalendarWidget component for clean separation
        let calendar_widget = CalendarWidget::new(self.selected_date, self.tasks.clone());
        calendar_widget.render(f, area);
    }
}
